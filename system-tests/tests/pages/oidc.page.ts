import type { Locator, Page } from "@playwright/test";
import { test } from "@playwright/test";
import BasePage from "./base.page";

export class OidcPage extends BasePage {
  locators: {
    continueButton: Locator;
    emailInput: Locator;
    loginButton: Locator;
    passwordInput: Locator;
  };

  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(page, `${process.env.OIDC_PROVIDER_URL}/interaction`);
    this.locators = {
      continueButton: page.getByRole('button', { name: 'Continue' }),
      loginButton: page.getByRole('button', { name: 'Sign-in' }),
      emailInput: page.getByRole('textbox', { name: 'login' }),
      passwordInput: page.getByRole('textbox', { name: 'password' })
    };
  }

  public async assertRedirectedTo(): Promise<void> {
    await test.step(`I should be redirected to the ${this.url} page`, async () => {
      await this.page.waitForURL((url) => url.href.startsWith(this.url));
    });
  }
}
