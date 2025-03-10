import type { Locator, Page } from "@playwright/test";
import { test } from "@playwright/test";
import BasePage from "./base.page";
import type { User } from "../base";

export class SignupPage extends BasePage {
  /**
   * The form email input.
   */
  private emailInput: Locator;

  /**
   * The form firstname input.
   */
  private firstnameInput: Locator;

  /**
   * The form lastname input.
   */
  private lastnameInput: Locator;

  /**
   * The form password input.
   */
  private passwordInput: Locator;

  /**
   * The signin link locator.
   */
  public signinLink: Locator;

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
    super(page, "/auth/subscribe");
    this.emailInput = page.locator("input#email");
    this.firstnameInput = page.locator("input#firstname");
    this.lastnameInput = page.locator("input#lastname");
    this.passwordInput = page.locator("input#password");
    this.signinLink = page.locator("[data-pw=login-link]");
    this.submitButton = page.locator("button[type=submit]");
    this.user = user;
  }

  /**
   * The signup form action
   */
  public form = {
    /**
     * Register the given user, defaulting to the fixtured default.
     */
    fill: async (
      user: Pick<User, "email" | "password" | "lastname" | "firstname"> = this
        .user,
    ): Promise<void> => {
      await test.step(`I fill the signup form as "${user.email}"`, async () => {
        await this.lastnameInput.click();
        await this.lastnameInput.fill(user.lastname);
        await this.firstnameInput.click();
        await this.firstnameInput.fill(user.firstname);
        await this.emailInput.click();
        await this.emailInput.fill(user.email);
        await this.passwordInput.click();
        await this.passwordInput.fill(user.password);
        await this.submitButton.click();
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
