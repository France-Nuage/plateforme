import { test } from "../../base";

test.describe('Security', () => {
  test('I am redirected to the OIDC provider when visiting a protected page as a guest', async ({ pages }) => {
    await pages.home.goto();
    await pages.oidc.assertRedirectedTo();
  });

  test('I am redirected to the home page when visiting a guest page as a user', async ({ actingAs, pages }) => {
    await actingAs();
    await pages.login.goto();
    await pages.managedServices.assertRedirectedTo();
  });

  test('I can authenticate with a valid user', async ({ keycloak, organization, services, pages }) => {
    // Provisionne un utilisateur complet via l'API (compte IdP + invitation dans
    // l'organisation de test), puis effectue le login par formulaire. On teste ici
    // qu'un utilisateur valide traverse le flux BFF confidentiel de bout en bout et
    // atteint la page des services managés.
    const password = 'password';
    const { access_token } = await keycloak.createUser({ password });
    const { email, preferred_username } = await keycloak.getUserInfo(access_token);
    await services.invitation.create({ organizationSlug: organization.slug, email });
    await pages.login.goto();
    await pages.oidc.login(preferred_username, password);
    await pages.managedServices.assertRedirectedTo();
  });
});
