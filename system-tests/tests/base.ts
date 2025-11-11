import { test as base } from "@playwright/test";
import { minBy } from "lodash";
import { configureResolver, instance, transport, Instance, KeyCloakApi, Organization, Project, ServiceMode, Services, Hypervisor, Zone } from "@france-nuage/sdk";
import { User } from '@/types';
import { InstancesPage, CreateInstancePage, HomePage, LoginPage, OidcPage } from "./pages";

/**
 * The fixtures exposed in the tests.
 */
type TestFixtures = {
  pages: {
    compute: {
      createInstance: CreateInstancePage;
      instances: InstancesPage;
    };
    oidc: OidcPage;
    home: HomePage;
    login: LoginPage;
  };

  /**
   * Acts as the requested user.
   *
   * This function:
   * 1. creates a user on the controlplane,
   * 2. invites the created user to the test organization,
   * 3. logs the created user in into the web console through session storage,
   * 4. instantiates an authenticated `Services` and returns it.
   */
  actingAs: (user?: Partial<User>) => Promise<Services>;
}

/**
 * The worker-scoped fixtures exposed in the tests.
 */
type WorkerFixtures = {
  admin: string;

  /**
   * Provides the test hypervisor.
   */
  hypervisor: Hypervisor;

  /**
   * Provides a `KeycloakApi` instance.
   */
  keycloak: KeyCloakApi;

  /**
   * Create an instance matching the given data.
   *
   * The instance will then be destroyed on 
   */
  instance: (instance: Partial<Instance>) => Promise<Instance>;

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

  /**
   * Generate a fresh proxmox instance to be used as a hypervisor.
   */
  proxmox: Instance;

  /**
   * Provides the controlplane services.
   */
  services: Services;

  /**
   * Provides a zone scoped for the test suite.
   */
  zone: Zone;
};

export const test = base.extend<TestFixtures, WorkerFixtures>({
  /**
   * @inheritdoc
   */
  actingAs: async ({ keycloak, organization, page, services }, use) => {
    await use(async (user) => {
      // compute key/value pair for session storage representation of the user
      await new Promise(resolve => setTimeout(resolve, 3000));
      const key = `oidc.user:${process.env.OIDC_PROVIDER_URL}:${process.env.OIDC_CLIENT_ID}`;
      const payload = await keycloak.createUser(user);
      const userinfo = await keycloak.getUserInfo(payload.access_token);
      console.log(`attempting to invite user on organization ${organization.id}`)
      await services.invitation.create({ organizationId: organization.id, email: userinfo.email });

      // define the session storage value in the context of the page
      await page.addInitScript(([key, value]) => sessionStorage.setItem(key, value), [key, JSON.stringify(payload)]);

      return configureResolver(transport('https://controlplane.test', payload.access_token))[ServiceMode.Rpc];
    });
  },

  /** 
   * @inheritdoc 
   */
  pages: async ({ page }, use) => use({
    compute: {
      createInstance: new CreateInstancePage(page),
      instances: new InstancesPage(page),
    },
    oidc: new OidcPage(page),
    home: new HomePage(page),
    login: new LoginPage(page),
  }),

  /**
   * @inheritdoc
   */
  hypervisor: [async ({ organization, proxmox, services, zone }, use) => {
    let { url, authorizationToken } = templates[proxmox.name as keyof typeof templates];

    let hypervisor = await services.hypervisor.register({
      url,
      authorizationToken,
      organizationId: organization.id,
      storageName: 'local-lvm',
      zoneId: zone.id,
    });

    use(hypervisor);
  }, { scope: 'worker' }],

  /**
   * @inheritdoc
   */
  instance: [async ({ project, services }, use) => {
    use((data: Partial<Instance>) => services.instance.create({
      ...data,
      ...instance(),
      image: 'debian-12-genericcloud-amd64-20250316-2053.qcow2',
      snippet: '',
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
    const rootOrganization = (await services.organization.list()).find((organization) => organization.name === (process.env.ROOT_ORGANIZATION_NAME ?? 'acme'));
    services.organization.create({ name: 'ACME', parentId: rootOrganization?.id }).then(use);
  }, { scope: 'worker' }],

  /**
   * @inheritdoc
   */
  production: [async ({ }, use) => {
    if (!process.env.PRODUCTION_CONTROLPLANE_TOKEN) {
      throw new Error('missing env var PRODUCTION_CONTROLPLANE_TOKEN');
    }
    const services = configureResolver(transport('https://controlplane.france-nuage.fr', process.env.PRODUCTION_CONTROLPLANE_TOKEN))[ServiceMode.Rpc];

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
  proxmox: [async ({ production }, use) => {
    if (!process.env.ROOT_SERVICE_ACCOUNT_KEY) {
      throw new Error('missing env var ROOT_SERVICE_ACCOUNT_KEY');
    }
    // Retrieve or register the dev hypervisor, which holds the test hypervisor instance template

    // Elect a proxmox template to use an instantiated hypervisor
    console.log('before fetching instances');
    const instances = await production.instance.list();
    console.log('after fetching instances');
    const { template, instance } = elect(instances);

    // If there is an associated instance with the template, stop and delete it
    if (!!instance) {
      await production.instance.stop(instance!.id);
      await production.instance.remove(instance!.id);
    }

    // Clone, start and register the template as a hypervisor
    console.log(`attempting to clone ${template.id}`);
    const proxmox = await production.instance.clone(template.id);
    await new Promise(resolve => setTimeout(resolve, 10000));
    await production.instance.start(proxmox.id);

    await use(proxmox);

    // cleanup
    await production.instance.stop(proxmox.id);
    await production.instance.remove(proxmox.id);
  }, { scope: 'worker', timeout: 1200000 }],


  /**
   * @inheritdoc
   */
  services: [async ({ proxmox }, use) => {
    if (!process.env.ROOT_SERVICE_ACCOUNT_KEY) {
      throw new Error('missing env var ROOT_SERVICE_ACCOUNT_KEY');
    }

    if (!proxmox) {
      throw new Error('proxmox hypervisor required to interface with the controlplane');
    }

    const services = configureResolver(transport('https://controlplane.test', process.env.ROOT_SERVICE_ACCOUNT_KEY))[ServiceMode.Rpc];

    use(services);
  }, { scope: 'worker', timeout: 1200000 }],

  /**
   * @inheritdoc
   */
  zone: [async ({ services }, use) => {
    const zone = await services.zone.create({ name: 'ACME-Mesa' });
    use(zone);
  }, { scope: 'worker' }],
});

export { expect } from "@playwright/test";

const elect = (instances: Instance[]) => {
  // extract templates from the instances list.
  const templates = instances.filter((instance) => /^pve\d+-test\d+-template$/.test(instance.name));

  if (templates.length === 0) {
    throw new Error('no electable templates');
  }

  // create a dictionary of template-instance association
  const dictionary: Record<string, { template: Instance, instance?: Instance }> = templates.reduce((acc, curr) => ({
    ...acc,
    [curr.name]: {
      template: curr,
      instance: instances.find((instance) => instance.name === `Copy - of - VM - ${curr.name}`),
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

const templates = {
  'Copy-of-VM-pve01-test01-template': {
    url: 'https://pve01-test01.france-nuage.fr',
    authorizationToken: 'PVEAPIToken=root@pam!controlplane=a87c51cc-f02c-476a-9168-9504be1bed79',
  },
  'Copy-of-VM-pve02-test01-template': {
    url: 'pve02-test01.france-nuage.fr',
    authorizationToken: 'PVEAPIToken=root@pam!controlplane=3f6ea76f-6316-4b12-9812-a376f3cd9d16',
  },
  'Copy-of-VM-pve03-test01-template': {
    url: 'pve03-test01.france-nuage.fr',
    authorizationToken: 'PVEAPIToken=root@pam!controlplane=fdcd2d52-7f6c-4d46-b899-efa40baa4659',
  }
}
