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
};

export type CheckoutResult = {
  subscriptionId: string;
  checkoutUrl: string;
};
