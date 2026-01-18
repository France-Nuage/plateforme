import { test } from "@/tests/base";
import { DEFAULT_IMAGE, DEFAULT_SNIPPET, instance } from "@france-nuage/sdk";

test.describe('Compute / Instances', () => {
  test('I can delete an existing instance', async ({ actingAs, pages, page, project }) => {
    test.slow();
    const services = await actingAs();
    const fixture = instance();
    const existing = await services.instance.create({
      maxCpuCores: 1,
      maxDiskBytes: 10 * 1024 ** 3,
      maxMemoryBytes: 1 * 1024 ** 3,
      name: fixture.name,
      image: DEFAULT_IMAGE,
      projectId: project.id,
      snippet: DEFAULT_SNIPPET,
    });

    await pages.compute.instances.goto();
    await pages.compute.instances.assertSee(existing.name);

    const row = page.getByRole('row').filter({ hasText: existing.name });
    await row.waitFor();
    await row.getByRole('button', { name: 'remove instance' }).click();
    await pages.compute.instances.confirmDialog('Supprimer');
    await row.waitFor({ state: 'detached' });
  });
});
