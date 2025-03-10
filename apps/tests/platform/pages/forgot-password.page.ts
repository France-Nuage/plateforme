import type { Locator, Page } from "@playwright/test";
import { test } from "@playwright/test";
import BasePage from "./base.page";
import type { User } from "../base";

export class ForgotPasswordPage extends BasePage {
  /**
   * The default user to authenticate against.
   */
  private user: User;

  /**
   * The form email input.
   */
  private emailInput: Locator;

  /**
   * The form submit button.
   */
  private submitButton: Locator;

  /**
   * @inheritdoc
   */
  public constructor(page: Page, user: User) {
    super(page, "/auth/forgot-password");
    this.user = user;
    this.emailInput = this.page.locator("input#email");
    this.submitButton = this.page.locator("button[type=submit]");
  }

  public form = {
    /**
     * Fill the forgot-password form
     */
    fill: async (
      user: Pick<User, "email" | "password"> = this.user,
    ): Promise<void> => {
      await test.step(`I fill the form as "${user.email}"`, async () => {
        await this.emailInput.click();
        await this.emailInput.fill(this.user.email);
      });
    },
    /**
     * Submit the forgot-password form
     */
    submit: async (): Promise<void> => {
      await this.submitButton.click();
    },
  };
}
