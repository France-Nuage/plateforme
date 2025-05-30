import type { Locator, Page } from "@playwright/test";
import BasePage from "./base.page";

export class LoginPage extends BasePage {
  locators: {
    loginButton: Locator;
  };

  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(page, "/login");
    this.locators = {
      loginButton: page.getByRole('button', { name: 'Se connecter avec Gitlab' }),
    };
  }
}
