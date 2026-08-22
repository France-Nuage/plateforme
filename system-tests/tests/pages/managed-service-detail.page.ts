import type { Locator, Page } from "@playwright/test";
import { test } from "@playwright/test";
import BasePage from "./base.page";

/**
 * Page object for the managed service detail page
 * (`/managed-services/:slug`), from which a user selects a plan and billing
 * period, picks a version, and clicks "Deployer".
 *
 * A paid plan (`requiresPayment`) triggers a Stripe Checkout redirect; a free
 * plan provisions immediately and redirects to the instances list. This POM
 * exposes the pieces the payment E2E drives: the billing-period toggle, plan
 * selection, and the deploy action.
 */
export class ManagedServiceDetailPage extends BasePage {
  public locators: {
    /** The "Plan" section heading, used to wait for plans to render. */
    planHeading: Locator;
    /** The "Deployer" submit button (note: the console label has no accent). */
    deployButton: Locator;
  };

  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(
      page,
      "/managed-services/:slug",
      new RegExp(`^${process.env.CONSOLE_URL}/managed-services/[^/]+(\\?.*)?$`),
    );

    this.locators = {
      planHeading: page.getByRole("heading", { name: "Plan", exact: true }),
      deployButton: page.getByRole("button", { name: "Deployer" }),
    };
  }

  /**
   * Select the billing period (monthly/yearly) via the segmented control.
   *
   * The console renders a Chakra `SegmentGroup` whose items are labelled
   * "Mensuel" (monthly) and "Annuel" (yearly).
   */
  public async selectBillingPeriod(period: "monthly" | "yearly"): Promise<void> {
    const label = period === "monthly" ? "Mensuel" : "Annuel";
    await test.step(`I select the ${label} billing period`, async () => {
      await this.page.getByText(label, { exact: true }).click();
    });
  }

  /**
   * Select a plan by its displayed name.
   *
   * Each plan is a clickable card whose title is a heading carrying the plan
   * name. The plan name can also appear as the page title (the service and its
   * plan often share a name, e.g. "GitLab Runner"), so we scope the click to
   * the plan cards under the "Plan" section: we take the plan-name heading that
   * sits after the "Plan" heading and click its card ancestor.
   */
  public async selectPlan(name: string): Promise<void> {
    await test.step(`I select the "${name}" plan`, async () => {
      // The plan-name heading inside a card is preceded by the "Plan" section
      // heading; `.last()` disambiguates it from the page-title heading, which
      // renders before the "Plan" section. Clicking the heading bubbles to the
      // card's onSelect handler.
      await this.page
        .getByRole("heading", { name, exact: true })
        .last()
        .click();
    });
  }

  /**
   * Fill a field of the auto-generated deploy form by its placeholder.
   *
   * The deploy form is rendered by RJSF (Chakra widgets) from the version's
   * configurable-values schema. We target inputs by their placeholder — set via
   * the version's UI schema (`ui:placeholder`) — rather than the label: Chakra's
   * Field associates label and input in a way Playwright's getByLabel does not
   * reliably resolve, whereas the placeholder sits directly on the input.
   */
  public async fillDeployValueByPlaceholder(
    placeholder: string,
    value: string,
  ): Promise<void> {
    await test.step(`I fill the field with placeholder "${placeholder}" in the deploy form`, async () => {
      await this.page.getByPlaceholder(placeholder).fill(value);
    });
  }

  /**
   * Click "Deployer".
   *
   * For a paid plan this creates a Stripe Checkout session and the browser is
   * redirected to the Stripe-hosted checkout page; for a free plan it
   * provisions immediately. The caller is responsible for awaiting the
   * subsequent navigation.
   */
  public async deploy(): Promise<void> {
    await test.step("I click Deployer", async () => {
      await this.locators.deployButton.click();
    });
  }
}
