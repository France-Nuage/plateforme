import { test as base } from "@playwright/test";
import { configureResolver, transport, KeyCloakApi, KubernetesCluster, ManagedServiceVersion, Organization, Project, ServiceMode, Services, organization } from "@france-nuage/sdk";
import { User } from '@/types';
import { HomePage, LoginPage, ManagedInstancesPage, ManagedServiceDetailPage, ManagedServicesPage, OidcPage } from "./pages";

/**
 * Slug of the managed service the deployment E2E exercises. Vaultwarden is
 * self-contained (no external token to become healthy), which makes it a
 * reliable target for "deploy and check it comes up".
 */
const DEPLOYABLE_SERVICE_SLUG = "vaultwarden";

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
    managedInstances: ManagedInstancesPage;
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

  /**
   * Ensures the GitLab Runner managed service has a registered version.
   *
   * A managed service is only deployable once a version is registered for it
   * (chart coordinates + the schemas that drive the deploy form). Production
   * registers versions from the charts pipeline; the ephemeral test environment
   * has none, so the payment E2E registers the version it needs here — a
   * precondition, like seeding an organization or a project. Idempotent
   * server-side, so concurrent workers/pipelines don't collide.
   */
  managedServiceVersion: ManagedServiceVersion;

  /**
   * Ensures a healthy hosting cluster exists for managed-service deployment.
   *
   * Deploying a managed service resolves a healthy cluster whose labels match
   * the service `deploy_target` (e.g. `availability=fr`). Real environments
   * register their fleet; the ephemeral test environment has none, so the
   * payment E2E enrols the qualif cluster it already runs on (via the injected
   * kubeconfig) and labels it accordingly. Cluster registration is a
   * platform-admin operation, so this acts as the bootstrap admin
   * (ROOT_ADMIN_EMAIL, seeded at control-plane startup).
   */
  deployCluster: KubernetesCluster;
};

export const test = base.extend<TestFixtures, WorkerFixtures>({
  /**
   * @inheritdoc
   */
  actingAs: async ({ keycloak, organization, pages, services }, use) => {
    await use(async (user) => {
      // Provision the user with an explicit password so the browser can complete
      // the real OIDC login below (the direct-grant token minting keeps working
      // for the returned RPC client).
      const password = 'password';
      const payload = await keycloak.createUser({ ...user, password });
      const userinfo = await keycloak.getUserInfo(payload.access_token);
      console.log(`attempting to invite user ${userinfo.email} on organization ${organization.slug}`)
      await services.invitation.create({ organizationSlug: organization.slug, email: userinfo.email });

      // Confidential-client BFF: the console is authenticated by the httpOnly
      // frn_session cookie the control plane sets at /auth/callback, never by a
      // client-side token. Drive the real login flow so that cookie is set —
      // injecting sessionStorage no longer authenticates the browser. Waiting for
      // the managed-services page confirms the session is live before the test
      // proceeds.
      await pages.login.goto();
      await pages.oidc.login(userinfo.preferred_username, password);
      await pages.managedServices.assertRedirectedTo();

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
    managedInstances: new ManagedInstancesPage(page),
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

  /**
   * @inheritdoc
   */
  managedServiceVersion: [async ({ }, use) => {
    // The control plane discovers deployable versions from the chart registry at
    // startup and persists them, so a test just reads the version that is
    // already there rather than registering one.
    if (!process.env.CI_SERVICE_TOKEN) {
      throw new Error('missing env var CI_SERVICE_TOKEN');
    }
    const services = configureResolver(transport(controlplaneUrl(), process.env.CI_SERVICE_TOKEN))[ServiceMode.Rpc];

    // The version list is populated asynchronously in the background at boot,
    // and discovery walks every catalogue chart, so on a freshly deployed
    // environment it can take a while before our service appears. Poll for up to
    // three minutes rather than racing it.
    let version: ManagedServiceVersion | undefined;
    for (let attempt = 0; attempt < 90 && !version; attempt++) {
      const versions = await services.managedService.listVersions(DEPLOYABLE_SERVICE_SLUG);
      version = versions[0];
      if (!version) {
        await new Promise((resolve) => setTimeout(resolve, 2000));
      }
    }
    if (!version) {
      throw new Error(
        `no deployable version found for service "${DEPLOYABLE_SERVICE_SLUG}" (control plane discovery not ready?)`,
      );
    }

    use(version);
    // Discovery runs in the background and walks every chart, so give this
    // fixture room beyond the default 30s per-fixture budget to poll for it.
  }, { scope: 'worker', timeout: 200_000 }],

  /**
   * @inheritdoc
   */
  deployCluster: [async ({ keycloak }, use) => {
    // The kubeconfig of the cluster the ephemeral environment runs on, injected
    // by the deploy job (see helm system-tests job + deploy.yml).
    if (!process.env.DEPLOY_KUBECONFIG) {
      throw new Error('missing env var DEPLOY_KUBECONFIG');
    }
    // Cluster registration is platform-admin only. We act as the bootstrap admin
    // seeded at control-plane startup: create its IdP account with the same
    // email so, once signed in, the control plane matches it (by email) to the
    // admin row it seeded.
    const adminEmail = process.env.ROOT_ADMIN_EMAIL;
    if (!adminEmail) {
      throw new Error('missing env var ROOT_ADMIN_EMAIL');
    }

    const payload = await keycloak.createUser({ email: adminEmail, username: adminEmail });
    const admin = configureResolver(transport(controlplaneUrl(), payload.access_token))[ServiceMode.Rpc];

    // Label the cluster with the deploy_target of the service under test
    // (availability=fr). Reuse the label if a concurrent worker already created
    // it; labels are unique by key/value.
    const labelKey = 'availability';
    const labelValue = 'fr';
    const existing = (await admin.kubernetesCluster.listLabels()).find(
      (l) => l.key.toLowerCase() === labelKey && l.value.toLowerCase() === labelValue,
    );
    const label = existing ?? (await admin.kubernetesCluster.createLabel({ key: labelKey, value: labelValue }));

    // A unique name per worker avoids collisions across concurrent pipelines
    // sharing the cluster; registering the same physical cluster several times
    // is fine (deploy picks any healthy match).
    const cluster = await admin.kubernetesCluster.createCluster({
      name: `qualif-${Date.now()}-${Math.floor(Math.random() * 1e6)}`,
      description: 'Ephemeral test hosting cluster (system tests)',
      kubeconfig: process.env.DEPLOY_KUBECONFIG,
      labelIds: [label.id],
    });

    use(cluster);
  }, { scope: 'worker' }],
});

export { expect } from "@playwright/test";
