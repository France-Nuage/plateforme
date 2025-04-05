import type { Page } from "@playwright/test";
import { test, expect } from "@playwright/test";

export default abstract class BasePage {
  /**
   * The Playwright page object.
   */
  protected readonly page: Page;

  /**
   * The page url.
   */
  protected readonly url: string;

  /**
   * Construct a page object model instance.
   *
   * @see https://playwright.dev/docs/pom
   */
  protected constructor(page: Page, url: string) {
    this.page = page;
    this.url = url;
  }

  /**
   * Assert the user is on this page.
   *
   * This function does not wait for the location to be the current page, it
   * just asserts the url. If you need to wait for the user to be redirected to
   * the current page, use the `assertRedirectedTo` method.
   */
  public async assertLocation(): Promise<void> {
    await expect(this.page, `Unexpected URL, got ${this.page.url()}, expected ${this.url}`).toHaveURL(
      this.url,
    );
  }

  /**
   * Assert the user is redirected to this page.
   */
  public async assertRedirectedTo(): Promise<void> {
    await test.step(`I should be redirected to the ${this.url} page`, async () => {
      await this.page.waitForURL(this.url);
      await this.assertLocation();
    });
  }

  /**
   * Navigate to the pom concrete page.
   */
  public async goto(): Promise<void> {
    await test.step(`I visit the ${this.url} page`, async () => {
      await this.page.goto(this.url);
    });
  }
}
