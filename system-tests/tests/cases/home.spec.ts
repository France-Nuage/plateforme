import { test } from "../base";

test.describe('Home', () => {
  test('I can access the home page', async ({ actingAs, pages }) => {
    await actingAs('Wile E. Coyote');
    await pages.home.goto();
    await pages.home.assertLocation();
  });
});
