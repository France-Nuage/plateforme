import { test as base } from "@playwright/test";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { ComputePage } from "./pages/compute.page";
import { HypervisorsClient } from "../protocol/hypervisors.client";
import { InstancesClient } from "../protocol/instances.client";
import { Hypervisor } from "../protocol/hypervisors";
import { Instance } from "../protocol/instances";
import { minBy } from "lodash";

const requiredEnvVars = ['CONTROLPLANE_URL', 'PROXMOX_DEV_AUTHORIZATION_TOKEN', 'PROXMOX_DEV_STORAGE_NAME', 'PROXMOX_DEV_URL', 'PROXMOX_TEST_AUTHORIZATION_TOKEN', 'PROXMOX_TEST_STORAGE_NAME', 'PROXMOX_TEST_URL'];

for (const variable of requiredEnvVars) {
  if (!process.env[variable]) {
    throw new Error(`missing env var ${variable}`);
  }
}

/**
 * The fixtures exposed in the tests.
 */
type TestFixtures = {
  pages: {
    compute: ComputePage;
  };

}

/**
 * The worker-scoped fixtures exposed in the tests.
 */
type WorkerFixtures = {
  /**
   * Provides instantiated grpc clients.
   *
   * These clients are instantiated and authenticated against the test cluster
   * and are meant to provide access to the test cluster through the
   * controlplane service. It requires the test engine to be executed through
   * docker and have the controlplane service dependency running.
   */
  grpc: {
    hypervisors: HypervisorsClient,
    instances: InstancesClient,
  };
  /**
   * Provides the test hypervisor.
   *
   * This is an hypervisor instance retrieved from the controlplane service,
   * living in database and ready to be used in tests.
   */
  hypervisor: Hypervisor;
};

export const test = base.extend<TestFixtures, WorkerFixtures>({
  /** 
   * @inheritdoc 
   */
  pages: async ({ page }, use) => use({
    compute: new ComputePage(page),
  }),

  /**
   * @inheritdoc
   */
  grpc: [async ({ }, use) => {
    const transport = new GrpcWebFetchTransport({ baseUrl: process.env.CONTROLPLANE_URL!, format: 'binary' });
    const hypervisors = new HypervisorsClient(transport);
    const instances = new InstancesClient(transport);

    use({ hypervisors, instances });
  }, { auto: true, scope: 'worker' }],

  /**
   * @inheritdoc
   */
  hypervisor: [async ({ grpc }, use) => {
    // Retrieve or register the dev hypervisor, which holds the test hypervisor instance template
    console.log('retrieving or registering dev hypervisor...');
    let devHypervisor = await grpc.hypervisors.listHypervisors({}).response.then(({ hypervisors }) => hypervisors.find((hypervisor) => hypervisor.url === process.env.PROXMOX_DEV_URL));
    if (!devHypervisor) {
      devHypervisor = await grpc.hypervisors.registerHypervisor({
        authorizationToken: process.env.PROXMOX_DEV_AUTHORIZATION_TOKEN!,
        storageName: process.env.PROXMOX_DEV_STORAGE_NAME!,
        url: process.env.PROXMOX_DEV_URL!,
      }).response.then((response) => response.hypervisor!);
    }

    // Elect a proxmox template to use an instantiated hypervisor
    console.log('selecting a proxmox template to clone...');
    const { instances } = await grpc.instances.listInstances({}).response.catch((error) => {
      console.log('failed listing', error);
      throw error;
    });
    const { template, instance } = elect(instances);

    // If there is an associated instance with the template, stop and delete it
    if (!!instance) {
      console.log('removing previous clone...');
      await grpc.instances.stopInstance({ id: instance.id }).response;
      await grpc.instances.deleteInstance({ id: instance.id }).response;
    }

    // Clone, start and register the template as a hypervisor
    console.log('cloning the proxmox template...');
    const clone = await grpc.instances.cloneInstance({ id: template.id }).response;
    console.log('starting the proxmox clone...');
    await grpc.instances.startInstance({ id: clone.id }).response;

    console.log('registering the proxmox clone as a hypervisor...');
    const { hypervisor } = await grpc.hypervisors.registerHypervisor({
      authorizationToken: process.env.PROXMOX_TEST_AUTHORIZATION_TOKEN!,
      storageName: process.env.PROXMOX_TEST_STORAGE_NAME!,
      url: process.env.PROXMOX_TEST_URL!,
    }).response;

    console.log('setup done!')
    console.log("\n\n");
    await use(hypervisor!);
    console.log("\n\n");

    // cleanup
    console.log('tests done, cleaning up...');
    await grpc.instances.stopInstance({ id: clone.id }).response;
    await grpc.instances.deleteInstance({ id: clone.id }).response;
    await grpc.hypervisors.detachHypervisor({ id: hypervisor!.id });

  }, { auto: true, scope: 'worker', timeout: 1200000 }],
});

export { expect } from "@playwright/test";

const elect = (instances: Instance[]) => {
  // extract templates from the instances list.
  const templates = instances.filter((instance) => /^pve\d+-test\d+-template$/.test(instance.name));

  // create a dictionary of template-instance association
  const dictionary: Record<string, { template: Instance, instance?: Instance }> = templates.reduce((acc, curr) => ({
    ...acc,
    [curr.name]: {
      template: curr,
      instance: instances.find((instance) => instance.name === `Copy-of-VM-${curr.name}`),
    }
  }), {});

  // get the first template that does not have an associated instance, if any
  const emptySlot = Object.values(dictionary).find(({ instance }) => !instance);

  if (emptySlot) {
    return emptySlot;
  }

  // otherwise elect a template
  return minBy(Object.values(dictionary), ({ instance }) => instance!.updatedAt!.seconds)!
}

