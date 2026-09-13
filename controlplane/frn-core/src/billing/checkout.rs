//! Stripe Checkout Session creation.

use fabrique::Query;
use frn_crypto::CURRENT_KEY_VERSION;
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::authorization::{Authorize, Permission, Principal};
use crate::billing::{
    Billing, BillingError, BillingPeriod, BillingSubscription, CheckoutMetadata,
    PendingInstanceParams, StripeClient, SubscriptionStatus,
};
use crate::managed::{ManagedServiceError, ManagedServicePlan};
use crate::resourcemanager::Project;

pub struct CreateCheckoutRequest {
    pub project_slug: String,
    pub organization_slug: String,
    pub service_slug: String,
    pub version_id: Uuid,
    pub plan_id: Uuid,
    pub billing_period: BillingPeriod,
    pub user_values: Option<Value>,
    pub secret_values: Option<Value>,
    /// Declared seat count for a per-seat plan. Required (>= 1) when the plan's
    /// `pricing_model` is not `flat`; must be `None`/absent for a flat plan.
    pub seats: Option<u32>,
}

pub struct CheckoutResponse {
    pub subscription_id: Uuid,
    pub checkout_url: String,
}

impl<A: Authorize, S: StripeClient> Billing<A, S> {
    /// Creates a Stripe Checkout Session for a managed service subscription.
    ///
    /// Validates permissions, resolves the Stripe price, creates the customer
    /// if needed, and stores the pending instance parameters for later
    /// provisioning upon webhook confirmation.
    pub async fn create_checkout_session<P: Principal + Sync>(
        &self,
        principal: &P,
        conn: &mut PgConnection,
        request: CreateCheckoutRequest,
    ) -> Result<CheckoutResponse, BillingError> {
        self.managed
            .auth
            .can(principal)
            .perform(Permission::CreateInstance)
            .over::<Project>(&request.project_slug)
            .await
            .map_err(|e| BillingError::ManagedService(e.into()))?;

        let service = self
            .managed
            .find_service_by_slug(&request.service_slug)
            .await?;

        self.managed.resolve_deploy_cluster(&service).await?;

        let plan = self.managed.find_plan_by_id(request.plan_id).await?;

        if plan.service_id != service.id {
            return Err(BillingError::ManagedService(
                ManagedServiceError::PlanServiceMismatch {
                    plan_id: plan.id,
                    service_id: service.id,
                },
            ));
        }

        if plan.status != "active" {
            return Err(BillingError::ManagedService(
                ManagedServiceError::PlanNotActive(plan.slug.clone()),
            ));
        }

        let price_id = resolve_stripe_price(&plan, &request.billing_period)?;

        // Resolve the line-item quantity from the plan's pricing model. Flat
        // plans are always billed with quantity 1 and reject a seat count;
        // per-seat plans require a seat count of at least 1.
        let seats = resolve_seats(&plan, request.seats)?;
        let quantity = seats.unwrap_or(1);

        let customer = self
            .find_or_create_customer(&mut *conn, &request.organization_slug)
            .await?;

        let subscription_id = Uuid::new_v4();

        let metadata = CheckoutMetadata {
            subscription_id,
            plan_id: plan.id,
            version_id: request.version_id,
            project_slug: request.project_slug.clone(),
            organization_slug: request.organization_slug.clone(),
        };

        let session = self
            .stripe
            .create_checkout_session(
                &customer.stripe_customer_id,
                &price_id,
                quantity,
                metadata,
                &self.success_url,
                &self.cancel_url,
            )
            .await?;

        BillingSubscription::query()
            .insert()
            .set(BillingSubscription::ID, subscription_id)
            .set(BillingSubscription::CUSTOMER_ID, customer.id)
            .set(
                BillingSubscription::STRIPE_CHECKOUT_SESSION_ID,
                Some(session.session_id.clone()),
            )
            .set(BillingSubscription::PLAN_ID, plan.id)
            .set(
                BillingSubscription::STATUS,
                SubscriptionStatus::PendingPayment,
            )
            .set(BillingSubscription::BILLING_PERIOD, request.billing_period)
            .set(BillingSubscription::SEATS, seats.map(|n| n as i32))
            .returning()
            .first(&mut *conn)
            .await?;

        let envelope = request
            .secret_values
            .as_ref()
            .map(|v| {
                let plaintext = serde_json::to_vec(v)?;
                let aad = build_pending_aad(subscription_id, CURRENT_KEY_VERSION);
                frn_crypto::encrypt(&self.kek, &plaintext, &aad, CURRENT_KEY_VERSION)
                    .map_err(BillingError::from)
            })
            .transpose()?;

        let mut q = PendingInstanceParams::query()
            .insert()
            .set(PendingInstanceParams::SUBSCRIPTION_ID, subscription_id)
            .set(PendingInstanceParams::SERVICE_SLUG, request.service_slug)
            .set(PendingInstanceParams::VERSION_ID, request.version_id)
            .set(PendingInstanceParams::PROJECT_SLUG, request.project_slug)
            .set(
                PendingInstanceParams::ORGANIZATION_SLUG,
                request.organization_slug,
            )
            .set(PendingInstanceParams::USER_VALUES, request.user_values);

        if let Some(ref env) = envelope {
            q = q
                .set(
                    PendingInstanceParams::SECRET_CIPHERTEXT,
                    Some(env.ciphertext.clone()),
                )
                .set(PendingInstanceParams::SECRET_NONCE, Some(env.nonce.clone()))
                .set(
                    PendingInstanceParams::SECRET_DEK_CIPHERTEXT,
                    Some(env.dek_ciphertext.clone()),
                )
                .set(
                    PendingInstanceParams::SECRET_DEK_NONCE,
                    Some(env.dek_nonce.clone()),
                )
                .set(
                    PendingInstanceParams::SECRET_KEY_VERSION,
                    Some(env.key_version),
                )
                .set(
                    PendingInstanceParams::SECRET_ALGORITHM,
                    Some(frn_crypto::ALGORITHM.to_owned()),
                );
        }

        q.returning().first(&mut *conn).await?;

        tracing::info!(
            subscription_id = %subscription_id,
            checkout_session = %session.session_id,
            "checkout session created"
        );

        Ok(CheckoutResponse {
            subscription_id,
            checkout_url: session.url,
        })
    }
}

pub(crate) fn build_pending_aad(subscription_id: Uuid, key_version: i32) -> Vec<u8> {
    let mut aad = subscription_id.as_bytes().to_vec();
    aad.extend_from_slice(&key_version.to_be_bytes());
    aad
}

fn resolve_stripe_price(
    plan: &ManagedServicePlan,
    period: &BillingPeriod,
) -> Result<String, BillingError> {
    if !plan.requires_payment {
        return Err(BillingError::PlanRequiresNoPayment(plan.slug.clone()));
    }

    let price = match period {
        BillingPeriod::Monthly => &plan.stripe_price_id_monthly,
        BillingPeriod::Yearly => &plan.stripe_price_id_yearly,
    };

    price
        .clone()
        .ok_or_else(|| BillingError::MissingStripePrice {
            plan_slug: plan.slug.clone(),
            period: *period,
        })
}

/// Validates and resolves the seat count against the plan's pricing model.
///
/// Returns the seat count to persist on the subscription (and to bill as the
/// line-item quantity):
/// - flat plan: `None` — a seat count is rejected as not applicable, and the
///   line item is billed with quantity 1;
/// - per-seat plan (`per_unit` / `tiered`): the requested seat count, which must
///   be present and at least 1.
///
/// # Errors
/// [`BillingError::SeatsRequired`] when a per-seat plan is missing a valid seat
/// count; [`BillingError::SeatsNotApplicable`] when a flat plan is given one.
fn resolve_seats(
    plan: &ManagedServicePlan,
    requested: Option<u32>,
) -> Result<Option<u32>, BillingError> {
    let requires_seats = plan.pricing_model != "flat";
    match (requires_seats, requested) {
        (true, Some(n)) if n >= 1 => Ok(Some(n)),
        (true, _) => Err(BillingError::SeatsRequired {
            plan_slug: plan.slug.clone(),
        }),
        (false, None) => Ok(None),
        (false, Some(_)) => Err(BillingError::SeatsNotApplicable {
            plan_slug: plan.slug.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::resolve_seats;
    use crate::billing::BillingError;
    use crate::managed::ManagedServicePlan;

    /// Builds a plan with the given pricing model; other fields are irrelevant
    /// to seat resolution.
    fn plan(pricing_model: &str) -> ManagedServicePlan {
        ManagedServicePlan {
            id: Uuid::new_v4(),
            service_id: Uuid::new_v4(),
            slug: "plan".to_owned(),
            name: "Plan".to_owned(),
            description: None,
            status: "active".to_owned(),
            highlighted: false,
            values_override: None,
            entitlements: serde_json::json!([]),
            price_monthly_cents: None,
            price_yearly_cents: None,
            stripe_price_id_monthly: None,
            stripe_price_id_yearly: None,
            requires_payment: true,
            pricing_model: pricing_model.to_owned(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn flat_plan_resolves_no_seats() {
        // Arrange, Act
        let resolved = resolve_seats(&plan("flat"), None).unwrap();

        // Assert
        assert_eq!(resolved, None);
    }

    #[test]
    fn flat_plan_rejects_seats() {
        // Arrange, Act
        let result = resolve_seats(&plan("flat"), Some(5));

        // Assert
        assert!(matches!(
            result,
            Err(BillingError::SeatsNotApplicable { .. })
        ));
    }

    #[test]
    fn per_seat_plan_resolves_requested_seats() {
        // Arrange, Act
        let resolved = resolve_seats(&plan("per_unit"), Some(10)).unwrap();

        // Assert
        assert_eq!(resolved, Some(10));
    }

    #[test]
    fn tiered_plan_resolves_requested_seats() {
        // Arrange, Act
        let resolved = resolve_seats(&plan("tiered"), Some(250)).unwrap();

        // Assert
        assert_eq!(resolved, Some(250));
    }

    #[test]
    fn per_seat_plan_rejects_missing_seats() {
        // Arrange, Act
        let result = resolve_seats(&plan("per_unit"), None);

        // Assert
        assert!(matches!(result, Err(BillingError::SeatsRequired { .. })));
    }

    #[test]
    fn per_seat_plan_rejects_zero_seats() {
        // Arrange, Act
        let result = resolve_seats(&plan("per_unit"), Some(0));

        // Assert
        assert!(matches!(result, Err(BillingError::SeatsRequired { .. })));
    }
}
