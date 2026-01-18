import { test, expect } from "@/tests/base";
import { DEFAULT_IMAGE, DEFAULT_SNIPPET, instance, project } from "@france-nuage/sdk";

test.describe('Compute / Instances', () => {
  test('I can move an instance to another project', async ({ actingAs, organization, pages, page, project: defaultProject, services }) => {
    test.slow();

    // Act as a user and get the services
    const userServices = await actingAs();
    const fixture = instance();

    // Create a second project in the same organization
    const targetProjectFixture = project();
    const targetProject = await services.project.create({
      name: targetProjectFixture.name,
      organizationId: organization.id,
    });

    // Create an instance in the default project
    const existing = await userServices.instance.create({
      maxCpuCores: 1,
      maxDiskBytes: 10 * 1024 ** 3,
      maxMemoryBytes: 1 * 1024 ** 3,
      name: fixture.name,
      image: DEFAULT_IMAGE,
      projectId: defaultProject.id,
      snippet: DEFAULT_SNIPPET,
    });

    // Navigate to instances page and verify the instance is visible
    await pages.compute.instances.goto();
    await pages.compute.instances.assertSee(existing.name);

    // Find the row and click on the "move to project" button
    const row = page.getByRole('row').filter({ hasText: existing.name });
    await row.waitFor();
    await row.getByRole('button', { name: 'move to project' }).click();

    // Wait for the dialog to appear
    const dialog = page.getByRole('dialog');
    await dialog.waitFor();

    // Click on the Select trigger to open the dropdown (Chakra UI Select)
    await dialog.getByText('Sélectionner un projet').click();

    // Wait for the dropdown content to appear and click the target project
    await page.getByText(new RegExp(targetProject.name)).click();

    // Click the "Déplacer" button
    await dialog.getByRole('button', { name: 'Déplacer' }).click();

    // Wait for the dialog to close
    await dialog.waitFor({ state: 'detached' });

    // Verify the instance was moved by checking the API
    const instances = await userServices.instance.list();
    const movedInstance = instances.find(i => i.id === existing.id);

    expect(movedInstance, 'Instance should exist after move').toBeDefined();
    expect(movedInstance!.projectId, 'Instance should be in target project').toBe(targetProject.id);
  });
});
