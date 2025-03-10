import type { Locator, Page } from "@playwright/test";
import { test } from "@playwright/test";
import BasePage from "./base.page";
import type { User } from "../base";

export class LoginPage extends BasePage {
  /**
   * The form email input.
   */
  private emailInput: Locator;

  /**
   * The 'forgot password' link locator.
   */
  public passwordForgottenLink: Locator;

  /**
   * The form password input.
   */
  private passwordInput: Locator;

  /**
   * The signup link locator.
   */
  public signupLink: Locator;

  /**
   * The form submit button.
   */
  private submitButton: Locator;

  /**
   * The default user to authenticate against.
   */
  private user: User;

  /**
   * @inheritdoc
   */
  public constructor(page: Page, user: User) {
    super(page, "/auth/login");
    this.user = user;
    this.emailInput = page.locator("input#email");
    this.passwordInput = page.locator("input#password");
    this.passwordForgottenLink = page.locator("[data-pw=forgot-password-link]");
    this.signupLink = page.locator("[data-pw=subscribe-link]");
    this.submitButton = page.locator("button[type=submit]");
  }

  /**
   * The login form actions.
   */
  public form = {
    /**
     * Fill the form with the given user data.
     */
    fill: async (
      user: Pick<User, "email" | "password"> = this.user,
    ): Promise<void> => {
      await test.step(`I fill the login form as "${user.email}"`, async () => {
        await this.emailInput.click();
        await this.emailInput.fill(user.email);
        await this.passwordInput.click();
        await this.passwordInput.fill(user.password);
      });
    },

    /**
     * Submit the form
     */
    submit: async (): Promise<void> => {
      await test.step("I submit the form", async () => {
        await this.submitButton.click();
      });
    },
  };
}
