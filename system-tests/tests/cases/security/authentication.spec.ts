import { test } from "../../base";

test.describe('Security', () => {
  test('I am redirected to the login page when visiting a protected page', async ({ pages }) => {
    await pages.home.goto();
    await pages.login.assertRedirectedTo();
  });
});
