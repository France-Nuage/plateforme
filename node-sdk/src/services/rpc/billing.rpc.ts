import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { BillingSubscriptionProto } from '../../generated/rpc/billing';
import { BillingServiceClient } from '../../generated/rpc/billing.client';
import {
  BillingSubscription,
  CheckoutResult,
  CreateCheckoutInput,
} from '../../models';
import { BillingService } from '../api';

export class BillingRpcService implements BillingService {
  private client: BillingServiceClient;

  constructor(transport: GrpcWebFetchTransport) {
    this.client = new BillingServiceClient(transport);
  }

  public cancelSubscription(subscriptionId: string): Promise<void> {
    return this.client
      .cancelSubscription({ subscriptionId })
      .response.then(() => {});
  }

  public createCheckoutSession(
    data: CreateCheckoutInput,
  ): Promise<CheckoutResult> {
    return this.client
      .createCheckoutSession({
        projectSlug: data.projectSlug,
        organizationSlug: data.organizationSlug,
        serviceSlug: data.serviceSlug,
        versionId: data.versionId,
        planId: data.planId,
        billingPeriod: data.billingPeriod,
        userValues: data.userValues,
        secretValues: data.secretValues,
      })
      .response.then((res) => ({
        subscriptionId: res.subscriptionId,
        checkoutUrl: res.checkoutUrl,
      }));
  }

  public getSubscription(subscriptionId: string): Promise<BillingSubscription> {
    return this.client
      .getSubscription({ subscriptionId })
      .response.then(({ subscription }) => fromRpcSubscription(subscription!));
  }

  public listSubscriptions(
    organizationSlug: string,
  ): Promise<BillingSubscription[]> {
    return this.client
      .listSubscriptions({ organizationSlug })
      .response.then(({ subscriptions }) =>
        subscriptions.map(fromRpcSubscription),
      );
  }
}

function fromRpcSubscription(
  sub: BillingSubscriptionProto,
): BillingSubscription {
  return {
    id: sub.id,
    customerId: sub.customerId,
    stripeSubscriptionId: sub.stripeSubscriptionId,
    planId: sub.planId,
    instanceId: sub.instanceId,
    status: sub.status,
    billingPeriod: sub.billingPeriod,
    currentPeriodStart: sub.currentPeriodStart
      ? new Date(Number(sub.currentPeriodStart.seconds) * 1000).toISOString()
      : undefined,
    currentPeriodEnd: sub.currentPeriodEnd
      ? new Date(Number(sub.currentPeriodEnd.seconds) * 1000).toISOString()
      : undefined,
    canceledAt: sub.canceledAt
      ? new Date(Number(sub.canceledAt.seconds) * 1000).toISOString()
      : undefined,
    createdAt: sub.createdAt
      ? new Date(Number(sub.createdAt.seconds) * 1000).toISOString()
      : new Date().toISOString(),
  };
}
