import { test } from "../base";

test.describe('Home', () => {
  test('I am redirected to the instances page', async ({ actingAs, pages }) => {
    await actingAs('Wile E. Coyote');
    await pages.home.goto();
    await pages.compute.assertLocation();
  });
});
