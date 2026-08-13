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

  test('I can authenticate with a valid user', async ({ pages, services, organization }) => {
    // wile.coyote est un utilisateur préexistant du realm importé. Comme actingAs
    // le fait pour les comptes qu'il crée, on le rattache à l'organisation de test :
    // un utilisateur authentifié sans organisation n'atterrit pas sur la page des
    // services managés. On teste ici le login par formulaire d'un utilisateur valide.
    await services.invitation.create({
      organizationSlug: organization.slug,
      email: 'wile.coyote@acme.org',
    });
    await pages.login.goto();
    await pages.oidc.assertRedirectedTo();
    await pages.oidc.locators.emailInput.fill('wile.coyote');
    await pages.oidc.locators.passwordInput.fill('anvil');
    await pages.oidc.locators.loginButton.click();
    await pages.managedServices.assertRedirectedTo();
  });
});
