import { test as base } from "@playwright/test";
import { ComputePage } from "./pages/compute.page";

type Fixtures = {
  pages: {
    compute: ComputePage;
  };
};

export const test = base.extend<Fixtures>({
  pages: async ({ page }, use) => use({
    compute: new ComputePage(page),
  })
});

export { expect } from "@playwright/test";

