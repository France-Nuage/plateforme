import { ManagedInstanceStatus } from "@france-nuage/sdk";
import { expect, test } from "../../base";

/**
 * End-to-end payment flow for a paid managed service plan.
 *
 * Drives the real Stripe-hosted Checkout page (functional source of truth):
 * an authenticated user deploys the paid Vaultwarden plan, pays with the Stripe
 * test card 4242 4242 4242 4242, is redirected to the success URL, and we assert
 * — through the SDK, not the UI — that the subscription became active and the
 * managed instance was provisioned by the `checkout.session.completed` webhook.
 *
 * Vaultwarden is a self-contained service (no external token needed to become
 * healthy), so it is a reliable target: the scenario asserts that provisioning
 * starts, not that the app is fully functional.
 *
 * Requires billing to be enabled on the control plane (STRIPE_SECRET_KEY +
 * STRIPE_WEBHOOK_SECRET) and the Stripe webhook to be relayed to it (locally
 * via `stripe/dev-billing.sh`; in CI via the ephemeral `stripe listen`). Where
 * billing is disabled, deploying a paid plan cannot reach checkout, so this
 * spec is skipped rather than failed.
 */
test.describe("Managed services / Vaultwarden checkout", () => {
  // Billing is opt-in. Without it, deploying a paid plan can never reach Stripe
  // Checkout, so the flow under test does not exist — skip rather than fail. The
  // control plane deployment sets BILLING_ENABLED only when both Stripe secrets
  // are wired (locally via `stripe/dev-billing.sh`, in CI via the protected
  // STRIPE_* variables); its absence means billing is off.
  test.skip(
    !process.env.BILLING_ENABLED,
    "billing disabled (BILLING_ENABLED unset): no Stripe checkout to exercise",
  );

  // The paid plan under test, aligned with controlplane/catalog/catalog.yaml
  // (vaultwarden-pme: requires_payment, 50,00 EUR/month).
  const SERVICE_NAME = "Vaultwarden";
  const PLAN_NAME = "Vaultwarden PME";

  test("Paying for a plan activates the subscription and provisions the instance", async ({
    actingAs,
    organization,
    project,
    pages,
    page,
    // Precondition: the service must have a deployable version, otherwise the
    // deploy form cannot render. The control plane discovers it from the chart
    // registry at startup; the worker fixture reads it (see base.ts).
    managedServiceVersion,
    // Precondition: a healthy hosting cluster matching the service deploy_target
    // must exist, otherwise checkout fails to resolve a deploy cluster. The
    // worker fixture enrols the qualif cluster (availability=fr).
    deployCluster,
  }) => {
    // This scenario spans the full paid flow — navigating the console, the
    // Stripe-hosted Checkout (whose subscription payment can sit in a
    // "Processing" state for over a minute before redirecting), then polling the
    // SDK until the webhook activates the subscription and provisions the
    // instance (two 60s polls). The default 30s per-test budget is far too
    // small; give it six minutes so the slow, legitimately asynchronous steps
    // (redirect up to 120s + the two polls) all have room.
    test.setTimeout(360_000);

    const services = await actingAs();

    // 1. Reach the service detail page and its plans.
    await pages.managedServices.goto();
    await pages.managedServices.locators.heading.waitFor();
    await pages.managedServices.openService(SERVICE_NAME);
    await pages.managedServiceDetail.assertLocation();
    await pages.managedServiceDetail.locators.planHeading.waitFor();

    // 2. Select the paid monthly plan and deploy. Vaultwarden's deploy form
    //    exposes only optional fields (a "signups allowed" toggle with a
    //    default), so no value needs to be filled to exercise checkout.
    await pages.managedServiceDetail.selectBillingPeriod("monthly");
    await pages.managedServiceDetail.selectPlan(PLAN_NAME);
    await pages.managedServiceDetail.deploy();

    // 3. Complete the Stripe-hosted Checkout with the test card. Deploying a
    //    paid plan redirects the browser to checkout.stripe.com.
    await page.waitForURL(/checkout\.stripe\.com/, { timeout: 30_000 });
    await completeStripeCheckout(page);

    // 4. Stripe redirects back to the success URL (the instances list). The
    //    subscription payment lingers in a "Processing" state before
    //    redirecting — observed anywhere from ~40s to well over a minute against
    //    the sandbox — so allow a generous budget rather than the default 30s.
    await pages.managedInstances.assertRedirectedTo({ timeout: 120_000 });
    await expect(page).toHaveURL(/checkout=success/);

    // 5. Assert the functional outcome through the SDK (source of truth): the
    //    webhook activates the subscription and provisions the instance. Both
    //    are asynchronous, so poll until they settle.
    await expect
      .poll(
        async () => {
          const subscriptions = await services.billing.listSubscriptions(
            organization.slug,
          );
          return subscriptions.some((s) => s.status === "active");
        },
        {
          message: "the subscription should become active after payment",
          timeout: 60_000,
          intervals: [1_000, 2_000, 5_000],
        },
      )
      .toBe(true);

    await expect
      .poll(
        async () => {
          const instances = await services.managedService.listInstances(
            project.slug,
          );
          return instances.some(
            (instance) =>
              instance.status === ManagedInstanceStatus.Provisioning ||
              instance.status === ManagedInstanceStatus.Running,
          );
        },
        {
          message: "a managed instance should be provisioned after payment",
          timeout: 60_000,
          intervals: [1_000, 2_000, 5_000],
        },
      )
      .toBe(true);
  });
});

/**
 * Fills and submits the Stripe-hosted Checkout form with the universally
 * successful test card (4242 4242 4242 4242).
 *
 * Stripe Checkout renders card fields with stable element ids
 * (`#cardNumber`, `#cardExpiry`, `#cardCvc`) and a submit button carrying the
 * `.SubmitButton` class. Email and cardholder-name fields are only present in
 * some configurations, so they are filled best-effort.
 *
 * When several payment methods are enabled on the account (Card, SEPA, Klarna,
 * …), Checkout renders them as an accordion and the card fields only mount once
 * the "Card" method is selected. The visible accordion header is a zero-sized,
 * off-viewport element, but it is backed by a real radio input
 * (`#payment-method-accordion-item-title-card`) we can `check({ force: true })`
 * to expand the card section. When card is the only method, that radio is
 * absent and the fields are already mounted, so the step is skipped.
 */
async function completeStripeCheckout(page: import("@playwright/test").Page) {
  await test.step("I pay with the Stripe test card 4242", async () => {
    const email = page.locator("#email");
    if (await email.count()) {
      await email.fill("e2e@france-nuage.test");
    }

    // Reveal the card fields when multiple payment methods are offered.
    const cardRadio = page.locator("#payment-method-accordion-item-title-card");
    if (await cardRadio.count()) {
      await cardRadio.check({ force: true });
    }

    const cardNumber = page.locator("#cardNumber");
    await cardNumber.waitFor({ state: "visible", timeout: 30_000 });
    await cardNumber.fill("4242424242424242");
    await page.locator("#cardExpiry").fill("12/34");
    await page.locator("#cardCvc").fill("123");

    const name = page.locator("#billingName");
    if (await name.count()) {
      await name.fill("E2E Test");
    }

    // The submit button carries the `.SubmitButton` class in single-method
    // checkout and a stable test id in the multi-method (accordion) layout.
    const submit = page
      .locator(".SubmitButton")
      .or(page.getByTestId("hosted-payment-submit-button"));
    await submit.first().click();
  });
}
