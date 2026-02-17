import { test } from "../base";

test.describe('Home', () => {
  test('I am redirected to the instances page', async ({ actingAs, pages }) => {
    test.skip(!process.env['PRODUCTION_CONTROLPLANE_TOKEN'], 'Requires production access');
    await actingAs({ name: 'Wile E. Coyote' });
    await pages.home.goto();
    await pages.compute.instances.assertLocation();
  });
});
