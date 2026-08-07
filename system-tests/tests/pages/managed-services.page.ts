import type { Locator, Page } from "@playwright/test";
import { test } from "@playwright/test";
import BasePage from "./base.page";

export class ManagedServicesPage extends BasePage {
  locators: {
    heading: Locator;
  };

  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(page, "/managed-services");
    this.locators = {
      heading: page.getByRole('heading', { name: 'Services managés' }),
    };
  }

  /**
   * Open the detail page of the service matching the given name.
   */
  public async openService(name: string): Promise<void> {
    await test.step(`I open the ${name} managed service`, async () => {
      await this.page.getByRole('link', { name }).click();
    });
  }
}
