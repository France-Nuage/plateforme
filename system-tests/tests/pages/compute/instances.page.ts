import type { Locator, Page } from "@playwright/test";
import BasePage from "../base.page";

export class InstancesPage extends BasePage {
  locators: {
    createInstanceButton: Locator;
  };
  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(page, "/compute/instances");
    this.locators = {
      createInstanceButton: page.getByRole('link', { name: 'Créer une nouvelle instance' }),
    };
  }
}
