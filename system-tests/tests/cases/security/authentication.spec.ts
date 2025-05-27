import { test } from "../../base";

test.describe('Security', () => {
  test('I am redirected to the login page when visiting a protected page as a guest', async ({ pages }) => {
    await pages.home.goto();
    await pages.login.assertRedirectedTo();
  });

  test('I am redirected to the home page when visiting a guest page as a user', async ({ actingAs, pages }) => {
    await actingAs('Wile E. Coyote');
    await pages.login.goto();
    await pages.home.assertRedirectedTo();
  })
});
