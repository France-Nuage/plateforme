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
    super(page, `${process.env.OIDC_PROVIDER_URL}/protocol/openid-connect/auth`);
    this.locators = {
      continueButton: page.getByRole('button', { name: 'Continue' }),
      loginButton: page.getByRole('button', { name: 'Sign In' }),
      emailInput: page.locator('#username'),
      passwordInput: page.locator('#password')
    };
  }

  public async assertRedirectedTo(): Promise<void> {
    await test.step(`I should be redirected to the ${this.url} page`, async () => {
      await this.page.waitForURL((url) => url.href.includes('/protocol/openid-connect/auth'));
    });
  }
}
