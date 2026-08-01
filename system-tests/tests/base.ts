import { test as base } from "@playwright/test";
import { configureResolver, transport, KeyCloakApi, Organization, Project, ServiceMode, Services, organization } from "@france-nuage/sdk";
import { User } from '@/types';
import { HomePage, LoginPage, ManagedServiceDetailPage, ManagedServicesPage, OidcPage } from "./pages";

/**
 * Resolves the control plane gRPC-web endpoint from the environment, falling
 * back to the local development URL when unset.
 */
const controlplaneUrl = (): string =>
  process.env.CONTROLPLANE_URL || 'https://controlplane.test';

/**
 * The fixtures exposed in the tests.
 */
type TestFixtures = {
  pages: {
    oidc: OidcPage;
    home: HomePage;
    login: LoginPage;
    managedServices: ManagedServicesPage;
    managedServiceDetail: ManagedServiceDetailPage;
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
   * Provides the test project.
   *
   * This is a generated project to scope the relations for the test suite.
   */
  project: Project;

  /**
   * Provides the controlplane services.
   */
  services: Services;
};

export const test = base.extend<TestFixtures, WorkerFixtures>({
  /**
   * @inheritdoc
   */
  actingAs: async ({ keycloak, organization, page, services }, use) => {
    await use(async (user) => {
      // compute key/value pair for session storage representation of the user
      const key = `oidc.user:${process.env.OIDC_PROVIDER_URL}:${process.env.OIDC_CLIENT_ID}`;
      const payload = await keycloak.createUser(user);
      const userinfo = await keycloak.getUserInfo(payload.access_token);
      console.log(`attempting to invite user ${userinfo.email} on organization ${organization.slug}`)
      await services.invitation.create({ organizationSlug: organization.slug, email: userinfo.email });

      // define the session storage value in the context of the page
      await page.addInitScript(([key, value]) => sessionStorage.setItem(key, value), [key, JSON.stringify(payload)]);

      return configureResolver(transport(controlplaneUrl(), payload.access_token))[ServiceMode.Rpc];
    });
  },

  /** 
   * @inheritdoc 
   */
  pages: async ({ page }, use) => use({
    oidc: new OidcPage(page),
    home: new HomePage(page),
    login: new LoginPage(page),
    managedServices: new ManagedServicesPage(page),
    managedServiceDetail: new ManagedServiceDetailPage(page),
  }),

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
    const fixture = organization();
    services.organization.create({ name: fixture.name, parentSlug: rootOrganization?.slug }).then(use);
  }, { scope: 'worker' }],

  /**
   * @inheritdoc
   */
  project: [async ({ organization, services }, use) => {
    const projects = await services.project.list();
    const project = projects.find((project) => project.organizationSlug === organization.slug);
    if (!project) {
      throw new Error(`could not find default project for organization ${organization.slug}`);
    }
    use(project);
  }, { scope: 'worker' }],

  /**
   * @inheritdoc
   */
  services: [async ({ }, use) => {
    if (!process.env.ROOT_SERVICE_ACCOUNT_KEY) {
      throw new Error('missing env var ROOT_SERVICE_ACCOUNT_KEY');
    }

    const services = configureResolver(transport(controlplaneUrl(), process.env.ROOT_SERVICE_ACCOUNT_KEY))[ServiceMode.Rpc];

    use(services);
  }, { scope: 'worker' }],
});

export { expect } from "@playwright/test";
