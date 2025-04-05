import { test } from "../base";

test.describe('Compute', () => {
  test('I can access the compute page', async ({ page, pages }) => {
    await pages.compute.goto();
    await pages.compute.assertLocation();
  });
});
