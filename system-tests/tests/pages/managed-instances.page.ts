import type { Locator, Page } from "@playwright/test";
import BasePage from "./base.page";

/**
 * Page object for the managed instances list (`/managed-services/instances`).
 *
 * This is the Stripe Checkout success destination
 * (`/managed-services/instances?checkout=success`) and the page a free-plan
 * deploy redirects to. It lists the active project's managed instances in a
 * table (service name, release, namespace, status badge). The payment E2E uses
 * it to confirm the browser landed here after checkout; the authoritative
 * assertions on subscription/instance state are made through the SDK.
 */
export class ManagedInstancesPage extends BasePage {
  public locators: {
    /** The page heading, used to wait for the list to render. */
    heading: Locator;
  };

  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(page, "/managed-services/instances");
    this.locators = {
      heading: this.page.getByRole("heading", {
        name: "Instances de services managés",
      }),
    };
  }

  /**
   * Locate the table row for the instance of the given service by its
   * displayed service name (the "Service" column).
   */
  public row(serviceName: string): Locator {
    return this.page.getByRole("row").filter({ hasText: serviceName });
  }
}
