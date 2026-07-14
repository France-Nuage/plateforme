import {
  BillingSubscription,
  CheckoutResult,
  CreateCheckoutInput,
} from '../../models';

export interface BillingService {
  createCheckoutSession: (data: CreateCheckoutInput) => Promise<CheckoutResult>;
  getSubscription: (subscriptionId: string) => Promise<BillingSubscription>;
  listSubscriptions: (
    organizationSlug: string,
  ) => Promise<BillingSubscription[]>;
  cancelSubscription: (subscriptionId: string) => Promise<void>;
}
