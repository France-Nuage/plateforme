//! Subscription lifecycle management (status transitions, cancellation).

use chrono::{DateTime, Utc};
use fabrique::Query;
use fabrique::sql::operators::Direction;
use uuid::Uuid;

use crate::authorization::{Authorize, Permission, Principal};
use crate::billing::{
    Billing, BillingCustomer, BillingError, BillingSubscription, StripeClient, SubscriptionStatus,
};
use crate::resourcemanager::Organization;

impl<A: Authorize, S: StripeClient> Billing<A, S> {
    /// Lists all subscriptions for a given organization.
    pub async fn list_subscriptions(
        &self,
        organization_slug: &str,
    ) -> Result<Vec<BillingSubscription>, BillingError> {
        let customer = self.find_customer(organization_slug).await?;

        BillingSubscription::query()
            .select()
            .r#where(BillingSubscription::CUSTOMER_ID, "=", customer.id)
            .order_by(BillingSubscription::CREATED_AT, Direction::Desc)
            .get(&self.db)
            .await
            .map_err(Into::into)
    }

    /// Finds a subscription by its internal ID.
    pub async fn find_subscription(
        &self,
        subscription_id: Uuid,
    ) -> Result<BillingSubscription, BillingError> {
        BillingSubscription::query()
            .select()
            .r#where(BillingSubscription::ID, "=", subscription_id)
            .first(&self.db)
            .await?
            .ok_or_else(|| BillingError::SubscriptionNotFound(subscription_id.to_string()))
    }

    /// Finds a subscription by its Stripe subscription ID.
    pub async fn find_subscription_by_stripe_id(
        &self,
        stripe_subscription_id: &str,
    ) -> Result<BillingSubscription, BillingError> {
        BillingSubscription::query()
            .select()
            .r#where(
                BillingSubscription::STRIPE_SUBSCRIPTION_ID,
                "=",
                Some(stripe_subscription_id.to_owned()),
            )
            .first(&self.db)
            .await?
            .ok_or_else(|| BillingError::SubscriptionNotFound(stripe_subscription_id.to_owned()))
    }

    /// Updates subscription status. Used by webhook handlers.
    pub async fn update_subscription_status(
        &self,
        stripe_subscription_id: &str,
        new_status: SubscriptionStatus,
    ) -> Result<(), BillingError> {
        let subscription = self
            .find_subscription_by_stripe_id(stripe_subscription_id)
            .await?;

        validate_status_transition(subscription.status, new_status)?;

        BillingSubscription::query()
            .update()
            .set(BillingSubscription::STATUS, new_status)
            .r#where(BillingSubscription::ID, "=", subscription.id)
            .execute(&self.db)
            .await?;

        tracing::info!(
            subscription_id = %subscription.id,
            stripe_subscription_id = stripe_subscription_id,
            old_status = %subscription.status,
            new_status = %new_status,
            "subscription status updated"
        );

        Ok(())
    }

    /// Updates the billing period timestamps on a subscription (from invoice.paid).
    pub async fn update_subscription_period(
        &self,
        stripe_subscription_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<(), BillingError> {
        let subscription = self
            .find_subscription_by_stripe_id(stripe_subscription_id)
            .await?;

        BillingSubscription::query()
            .update()
            .set(
                BillingSubscription::CURRENT_PERIOD_START,
                Some(period_start),
            )
            .set(BillingSubscription::CURRENT_PERIOD_END, Some(period_end))
            .r#where(BillingSubscription::ID, "=", subscription.id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Cancels a subscription both in Stripe and locally.
    pub async fn cancel_subscription(&self, subscription_id: Uuid) -> Result<(), BillingError> {
        let subscription = self.find_subscription(subscription_id).await?;
        self.do_cancel_subscription(&subscription).await
    }

    async fn do_cancel_subscription(
        &self,
        subscription: &BillingSubscription,
    ) -> Result<(), BillingError> {
        if let Some(ref stripe_sub_id) = subscription.stripe_subscription_id {
            self.stripe.cancel_subscription(stripe_sub_id).await?;
        }

        BillingSubscription::query()
            .update()
            .set(BillingSubscription::STATUS, SubscriptionStatus::Canceled)
            .set(BillingSubscription::CANCELED_AT, Some(Utc::now()))
            .r#where(BillingSubscription::ID, "=", subscription.id)
            .execute(&self.db)
            .await?;

        tracing::info!(
            subscription_id = %subscription.id,
            "subscription canceled"
        );

        Ok(())
    }

    pub async fn get_subscription_checked<P: Principal + Sync>(
        &self,
        principal: &P,
        subscription_id: Uuid,
    ) -> Result<BillingSubscription, BillingError> {
        let subscription = self.find_subscription(subscription_id).await?;
        self.authorize_subscription(principal, &subscription, Permission::Get)
            .await?;
        Ok(subscription)
    }

    pub async fn list_subscriptions_checked<P: Principal + Sync>(
        &self,
        principal: &P,
        organization_slug: &str,
    ) -> Result<Vec<BillingSubscription>, BillingError> {
        let slug = organization_slug.to_owned();
        self.managed
            .auth
            .can(principal)
            .perform(Permission::List)
            .over::<Organization>(&slug)
            .await
            .map_err(|e| BillingError::ManagedService(e.into()))?;

        self.list_subscriptions(organization_slug).await
    }

    pub async fn cancel_subscription_checked<P: Principal + Sync>(
        &self,
        principal: &P,
        subscription_id: Uuid,
    ) -> Result<(), BillingError> {
        let subscription = self.find_subscription(subscription_id).await?;
        self.authorize_subscription(principal, &subscription, Permission::Delete)
            .await?;
        self.do_cancel_subscription(&subscription).await
    }

    async fn authorize_subscription<P: Principal + Sync>(
        &self,
        principal: &P,
        subscription: &BillingSubscription,
        permission: Permission,
    ) -> Result<(), BillingError> {
        let customer = BillingCustomer::query()
            .select()
            .r#where(BillingCustomer::ID, "=", subscription.customer_id)
            .first(&self.db)
            .await?
            .ok_or_else(|| BillingError::CustomerNotFound(subscription.customer_id.to_string()))?;

        self.managed
            .auth
            .can(principal)
            .perform(permission)
            .over::<Organization>(&customer.organization_slug)
            .await
            .map_err(|e| BillingError::ManagedService(e.into()))
    }
}

pub(super) fn validate_status_transition(
    from: SubscriptionStatus,
    to: SubscriptionStatus,
) -> Result<(), BillingError> {
    let valid = matches!(
        (from, to),
        (
            SubscriptionStatus::PendingPayment,
            SubscriptionStatus::Active
        ) | (
            SubscriptionStatus::PendingPayment,
            SubscriptionStatus::Canceled
        ) | (
            SubscriptionStatus::PendingPayment,
            SubscriptionStatus::Incomplete
        ) | (SubscriptionStatus::Active, SubscriptionStatus::PastDue)
            | (SubscriptionStatus::Active, SubscriptionStatus::Canceled)
            | (SubscriptionStatus::PastDue, SubscriptionStatus::Active)
            | (SubscriptionStatus::PastDue, SubscriptionStatus::Canceled)
            | (SubscriptionStatus::Incomplete, SubscriptionStatus::Active)
            | (SubscriptionStatus::Incomplete, SubscriptionStatus::Canceled)
    );

    if !valid {
        return Err(BillingError::InvalidStatusTransition { from, to });
    }

    Ok(())
}
