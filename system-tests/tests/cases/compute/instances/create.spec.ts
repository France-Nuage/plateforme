import { test } from "@/tests/base";

test.describe('Compute / Instances', () => {
  test('I can navigate to the "Create Instance" page from the instances list', async ({ actingAs, pages }) => {
    await actingAs();
    await pages.compute.instances.goto();
    await pages.compute.instances.locators.createInstanceButton.click();
    await pages.compute.createInstance.assertRedirectedTo();
  });
  test('I can create a new instance', async ({ actingAs, pages }) => {
    await actingAs();
    await pages.compute.createInstance.goto();
    await new Promise((resolve) => setTimeout(resolve, 30 * 1000));
  });
});

