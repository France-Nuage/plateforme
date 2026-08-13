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
    // Sélecteurs unifiés couvrant les deux IdP : l'env CI déploie Keycloak
    // (identifiant #username, bouton submit #kc-login rendu en <input type=submit>
    // "Sign In"), la prod FerrisKey (champ #email, <button type=submit>). Les deux
    // ne coexistent jamais sur une même page, donc l'union CSS reste stricte.
    this.locators = {
      continueButton: page.getByRole('button', { name: 'Continue' }),
      loginButton: page.locator('#kc-login, button[type="submit"]'),
      emailInput: page.locator('#username, #email'),
      passwordInput: page.locator('#password')
    };
  }

  public async assertRedirectedTo(): Promise<void> {
    await test.step(`I should be redirected to the ${this.url} page`, async () => {
      // Keycloak rend le formulaire de login directement sur l'authorize
      // (/protocol/openid-connect/auth) ; FerrisKey redirige vers la page de login
      // du webapp (/realms/<realm>/authentication/login).
      await this.page.waitForURL((url) =>
        url.href.includes('/authentication/login') ||
        url.href.includes('/protocol/openid-connect/auth'));
    });
  }

  /**
   * Completes the identity-provider login form and submits it.
   *
   * Waits for the authorize/login page, fills the credentials, then clicks the
   * submit button (which POSTs to the IdP and redirects back to the control-plane
   * BFF callback).
   */
  public async login(username: string, password: string): Promise<void> {
    await this.assertRedirectedTo();
    await this.locators.emailInput.fill(username);
    await this.locators.passwordInput.fill(password);
    await this.locators.loginButton.click();
  }
}
