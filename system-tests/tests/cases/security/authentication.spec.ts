import { createUser } from "@/oidc";
import { test } from "../../base";

test.describe('Security', () => {
  test('I am redirected to the login page when visiting a protected page as a guest', async ({ pages }) => {
    await pages.home.goto();
    await pages.login.assertRedirectedTo();
  });

  // test('I am redirected to the home page when visiting a guest page as a user', async ({ actingAs, pages }) => {
  //   await actingAs('Wile E. Coyote');
  //   await pages.login.goto();
  //   await pages.compute.assertRedirectedTo();
  // });

  test('I can authenticate with a valid user', async ({ page, pages }) => {
    // Use pre-existing Keycloak user from realm import
    await pages.login.goto();
    await pages.login.locators.loginButton.click();
    await pages.oidc.assertRedirectedTo();
    await pages.oidc.locators.emailInput.fill('wile.coyote');
    await pages.oidc.locators.passwordInput.fill('anvil');
    await pages.oidc.locators.loginButton.click();
    // Keycloak might skip consent screen for public clients
    await pages.compute.assertRedirectedTo();
  })
});
