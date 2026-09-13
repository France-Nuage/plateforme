export type BillingSubscription = {
  id: string;
  customerId: string;
  stripeSubscriptionId?: string;
  planId: string;
  instanceId?: string;
  status: string;
  billingPeriod: string;
  currentPeriodStart?: string;
  currentPeriodEnd?: string;
  canceledAt?: string;
  /**
   * Number of seats billed for a per-seat plan (FRA-15). Undefined for flat
   * plans, where the quantity is always 1.
   */
  seats?: number;
  createdAt: string;
};

export type CreateCheckoutInput = {
  projectSlug: string;
  organizationSlug: string;
  serviceSlug: string;
  versionId: string;
  planId: string;
  billingPeriod: 'monthly' | 'yearly';
  userValues?: string;
  secretValues?: string;
  /**
   * Number of seats to bill for a per-seat plan (FRA-15). Required (>= 1) when
   * the plan's pricing model is `per_unit` or `tiered`; omitted for flat plans.
   */
  seats?: number;
};

export type CheckoutResult = {
  subscriptionId: string;
  checkoutUrl: string;
};
