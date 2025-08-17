import { test } from "../../base";

test.describe('Security', () => {
  test('I am redirected to the login page when visiting a protected page as a guest', async ({ pages }) => {
    await pages.home.goto();
    await pages.login.assertRedirectedTo();
  });

  test('I am redirected to the home page when visiting a guest page as a user', async ({ actingAs, pages }) => {
    await actingAs('Wile E. Coyote');
    await pages.login.goto();
    await pages.compute.assertRedirectedTo();
  });

  test('I can authenticate with a valid user', async ({ page, pages }) => {
    await pages.login.goto();
    await pages.login.locators.loginButton.click();
    await pages.oidc.assertRedirectedTo();
    await pages.oidc.locators.emailInput.fill('wcoyote@acme.org');
    await pages.oidc.locators.passwordInput.fill('killbipbip');
    await pages.oidc.locators.loginButton.click();
    await pages.oidc.locators.continueButton.waitFor();
    await pages.oidc.locators.continueButton.click();
    await pages.compute.assertRedirectedTo();
  })
});
