import type { Page } from "@playwright/test";
import BasePage from "./base.page";

/**
 * Login page object model.
 *
 * This page automatically redirects to the OIDC provider.
 * No user interaction is required on this page.
 */
export class LoginPage extends BasePage {
  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(page, "/login");
  }
}
