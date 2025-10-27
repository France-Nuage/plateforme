import { test as base } from "@playwright/test";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { minBy } from "lodash";
import { configureResolver, instance, Instance, KeyCloakApi, Organization, Project, ServiceMode, Services } from "@france-nuage/sdk";
import { createUser } from "@/oidc";
import { User } from '@/types';
import { ComputePage, HomePage, LoginPage, OidcPage } from "./pages";

/**
 * The fixtures exposed in the tests.
 */
type TestFixtures = {
  pages: {
    compute: ComputePage;
    oidc: OidcPage;
    home: HomePage;
    login: LoginPage;
  };

  actingAs: (user?: Partial<User>) => Promise<void>;
}

/**
 * The worker-scoped fixtures exposed in the tests.
 */
type WorkerFixtures = {
  admin: string;

  /**
   * Provides the controlplane services.
   */
  services: Services;

  /**
   * Create an instance matching the given data.
   *
   * The instance will then be destroyed on 
   */
  instance: (instance: Partial<Instance>) => Promise<Instance>;

  /**
   * Provides a `KeycloakApi` instance.
   */
  keycloak: KeyCloakApi;

  /**
   * Provides the test organization.
   *
   * This is a generated organization to scope the relations for the test suite.
   */
  organization: Organization;

  /**
   * The production services.
   *
   * The production services are meant to be used by the fixtures **exclusively** in order to
   * to provision on the France Nuage cloud a dedicated hypervisor. this hypervisor is then
   * registered by the test engine into the local controlplane under test.
   *
   * The only location under which it should be called is the `local` fixture, which expose services
   * for the controlplane under test. Any other usage should be thoroughly investigated as it is a
   * smell of miss-use.
   */
  production: Services;

  /**
   * Provides the test project.
   *
   * This is a generated project to scope the relations for the test suite.
   */
  project: Project;
};

export const test = base.extend<TestFixtures, WorkerFixtures>({
  /**
   * @inheritdoc
   */
  actingAs: async ({ keycloak, page }, use) => {
    await use(async (user) => {
      // compute key/value pair for session storage representation of the user
      const key = `oidc.user:${process.env.OIDC_PROVIDER_URL}:${process.env.OIDC_CLIENT_ID}`;
      const value = await keycloak.createUser(user);
      // define the session storage value in the context of the page
      await page.addInitScript(([key, value]) => {
        console.log(`serializing under '${key}' ...`, value);
        sessionStorage.setItem(key, value)
      }, [key, JSON.stringify(value)]);
    });
  },

  /** 
   * @inheritdoc 
   */
  pages: async ({ page }, use) => use({
    compute: new ComputePage(page),
    oidc: new OidcPage(page),
    home: new HomePage(page),
    login: new LoginPage(page),
  }),

  /**
   * @inheritdoc
   */
  instance: [async ({ project, services }, use) => {
    use((data: Partial<Instance>) => services.instance.create({
      ...data,
      ...instance(),
      projectId: project.id,
    }));
  }, { scope: 'worker' }],

  /**
   * @inheritdoc
   */
  keycloak: [({ }, use) => {
    const url = process.env["KEYCLOAK_URL"] || 'https://keycloak.test';
    const admin = {
      username: process.env["KEYCLOAK_ADMIN"] || 'admin',
      password: process.env["KEYCLOAK_ADMIN_PASSWORD"] || 'admin',
    };

    use(new KeyCloakApi(url, admin));
  }, { scope: 'worker' }],

  /**
   * @inheritdoc
   */
  organization: [async ({ services }, use) => {
    services.organization.create({ name: 'ACME' }).then(use)
  }, { scope: 'worker' }],

  /**
   * @inheritdoc
   */
  production: [async ({ }, use) => {
    const transport = new GrpcWebFetchTransport({
      baseUrl: 'https://controlplane.france-nuage.fr', interceptors: [
        {
          interceptUnary(next, method, input, options) {
            return next(method, input, {
              ...options,
              meta: {
                ...options.meta,
                'Authorization': `Bearer ${process.env.PRODUCTION_CONTROLPLANE_TOKEN}`,
              }
            });
          }
        }
      ]
    });
    // @ts-ignore
    const services = configureResolver(transport)[ServiceMode.Rpc];

    await use(services);
  }, { scope: 'worker' }],

  /**
   * @inheritdoc
   */
  project: [async ({ organization, services }, use) => {
    services.project.create({
      name: 'Anvil Factory',
      organizationId: organization.id
    }).then(use);
  }, { scope: 'worker' }],

  /**
   * @inheritdoc
   */
  services: [async ({ production }, use) => {
    // Retrieve or register the dev hypervisor, which holds the test hypervisor instance template
    console.log('retrieving or registering dev hypervisor...');

    // Elect a proxmox template to use an instantiated hypervisor
    console.log('selecting a proxmox template to clone...');
    const instances = await production.instance.list();
    const { template, instance } = elect(instances);

    // If there is an associated instance with the template, stop and delete it
    if (!!instance) {
      console.log('removing previous clone...');
      await production.instance.stop(instance!.id);
      await production.instance.remove(instance!.id);
    }

    // Clone, start and register the template as a hypervisor
    console.log('cloning the proxmox template...');
    const clone = await production.instance.clone(template.id);
    console.log('starting the proxmox clone...');
    await production.instance.start(clone.id);

    console.log('registering the proxmox clone as a hypervisor...');
    const transport = new GrpcWebFetchTransport({ baseUrl: 'https://controlplane.test' });
    // @ts-ignore
    const services = configureResolver(transport)[ServiceMode.Rpc];

    await use(services);

    // cleanup
    console.log('\n\ntests done, cleaning up...');
    await services.instance.stop(clone.id);
    await services.instance.remove(clone.id);
  }, { scope: 'worker', timeout: 1200000 }],
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
  return minBy(Object.values(dictionary), ({ instance }) => instance!.updatedAt!)!
}
