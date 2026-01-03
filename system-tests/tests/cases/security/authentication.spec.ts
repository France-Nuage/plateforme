import { test } from "../../base";

test.describe('Security', () => {
  test('I am redirected to the OIDC provider when visiting a protected page as a guest', async ({ pages }) => {
    await pages.home.goto();
    await pages.oidc.assertRedirectedTo();
  });

  test('I am redirected to the home page when visiting a guest page as a user', async ({ actingAs, pages }) => {
    await actingAs();
    await pages.login.goto();
    await pages.compute.instances.assertRedirectedTo();
  });

  test('I can authenticate with a valid user', async ({ pages }) => {
    await pages.login.goto();
    await pages.oidc.assertRedirectedTo();
    await pages.oidc.locators.emailInput.fill('wile.coyote');
    await pages.oidc.locators.passwordInput.fill('anvil');
    await pages.oidc.locators.loginButton.click();
    await pages.compute.instances.assertRedirectedTo();
  });

  test('Login page automatically redirects to OIDC provider', async ({ pages }) => {
    await pages.login.goto();
    await pages.oidc.assertRedirectedTo();
  });
});
