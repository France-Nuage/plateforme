import { test } from "../base";

test.describe('Home', () => {
  test('I am redirected to the managed services page', async ({ actingAs, pages }) => {
    await actingAs({ name: 'Wile E. Coyote' });
    await pages.home.goto();
    await pages.managedServices.locators.heading.waitFor();
    await pages.managedServices.assertLocation();
  });
});
