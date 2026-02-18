import { test } from "@/tests/base";
import { instance } from "@france-nuage/sdk";

test.describe('Compute / Instances', () => {
  test('I can navigate to the "Create Instance" page from the instances list', async ({ actingAs, pages }) => {
    await actingAs();
    await pages.compute.instances.goto();
    await pages.compute.instances.locators.createInstanceButton.click();
    await pages.compute.createInstance.assertRedirectedTo();
  });

  test('I can create a new instance', async ({ actingAs, hypervisor, pages }) => {
    test.slow();
    const fixture = instance();
    await actingAs();
    await pages.compute.instances.goto();
    await pages.compute.instances.locators.createInstanceButton.click();
    await pages.compute.createInstance.assertRedirectedTo();

    await pages.compute.createInstance.locators.nameField.fill(fixture.name);
    await pages.compute.createInstance.locators.createInstanceButton.click();

    await pages.compute.instances.assertRedirectedTo({ timeout: 120 * 1000 });
    await pages.compute.instances.assertSee(fixture.name);
  });
});

