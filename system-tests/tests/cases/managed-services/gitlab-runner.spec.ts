import { test } from "../../base";

test.describe('Managed services / GitLab Runner', () => {
  test('I can browse to the GitLab Runner service and reach its plans', async ({ actingAs, pages }) => {
    await actingAs();
    await pages.managedServices.goto();
    await pages.managedServices.locators.heading.waitFor();

    await pages.managedServices.openService('GitLab Runner');
    await pages.managedServiceDetail.assertLocation();
  });
});
