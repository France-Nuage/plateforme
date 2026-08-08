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
    // Sélecteurs FerrisKey (webapp login form, source-vérifié front/login-form.tsx +
    // rendu live) : champ #email (type=email), #password, bouton submit. Diffère de
    // Keycloak (#username / bouton "Sign In").
    this.locators = {
      continueButton: page.getByRole('button', { name: 'Continue' }),
      loginButton: page.locator('button[type="submit"]'),
      emailInput: page.locator('#email'),
      passwordInput: page.locator('#password')
    };
  }

  public async assertRedirectedTo(): Promise<void> {
    await test.step(`I should be redirected to the ${this.url} page`, async () => {
      // FerrisKey redirige l'authorize (/protocol/openid-connect/auth) vers la page
      // de login du webapp (/realms/<realm>/authentication/login).
      await this.page.waitForURL((url) =>
        url.href.includes('/authentication/login') ||
        url.href.includes('/protocol/openid-connect/auth'));
    });
  }
}
