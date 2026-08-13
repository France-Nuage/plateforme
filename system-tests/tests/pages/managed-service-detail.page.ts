import type { Page } from "@playwright/test";
import BasePage from "./base.page";

export class ManagedServiceDetailPage extends BasePage {
  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(page, "/managed-services/:slug", new RegExp(`^${process.env.CONSOLE_URL}/managed-services/[^/]+(\\?.*)?$`));
  }
}
