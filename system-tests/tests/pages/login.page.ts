import type { Page } from "@playwright/test";
import BasePage from "./base.page";

export class LoginPage extends BasePage {
  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(page, "/login/");
  }
}
