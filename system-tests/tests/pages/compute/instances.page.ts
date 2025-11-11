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
    super(page, "/compute/instances", new RegExp(`^${process.env.CONSOLE_URL}/compute/instances(\\?.*)?`));
    this.locators = {
      createInstanceButton: page.getByRole('button', { name: 'Créer une nouvelle instance' }),
    };
  }
}
