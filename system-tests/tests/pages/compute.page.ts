import type { Page } from "@playwright/test";
import BasePage from "./base.page";

export class ComputePage extends BasePage {
  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(page, "/compute/instances", new RegExp(`^${process.env.CONSOLE_URL}/compute/instances(\\?.*)?`));
  }
}
