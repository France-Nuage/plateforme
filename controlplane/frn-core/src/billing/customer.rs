//! Stripe customer lifecycle tied to organizations.

use fabrique::Query;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::authorization::Authorize;
use crate::billing::{Billing, BillingCustomer, BillingError, StripeClient};
use crate::resourcemanager::Organization;

impl<A: Authorize, S: StripeClient> Billing<A, S> {
    /// Finds the billing customer for an organization, or creates one in Stripe
    /// and persists the mapping.
    ///
    /// Uses INSERT ... ON CONFLICT to handle concurrent requests for the same
    /// organization safely. If a race occurs, the losing request logs a warning
    /// about the orphaned Stripe customer and returns the existing row.
    pub async fn find_or_create_customer(
        &self,
        conn: &mut PgConnection,
        organization_slug: &str,
    ) -> Result<BillingCustomer, BillingError> {
        let existing = BillingCustomer::query()
            .select()
            .r#where(
                BillingCustomer::ORGANIZATION_SLUG,
                "=",
                organization_slug.to_owned(),
            )
            .first(&mut *conn)
            .await?;

        if let Some(customer) = existing {
            return Ok(customer);
        }

        let organization = Organization::query()
            .select()
            .r#where(Organization::SLUG, "=", organization_slug.to_owned())
            .first(&mut *conn)
            .await?
            .ok_or_else(|| {
                BillingError::Stripe(format!("organization not found: {}", organization_slug))
            })?;

        let stripe_customer_id = self
            .stripe
            .create_customer(organization_slug, &organization.name)
            .await?;

        // fabrique limitation: ON CONFLICT on non-PK unique constraint (organization_slug)
        let inserted = sqlx::query_as::<_, BillingCustomer>(
            r#"INSERT INTO billing.customer (id, organization_slug, stripe_customer_id)
               VALUES ($1, $2, $3)
               ON CONFLICT (organization_slug) DO NOTHING
               RETURNING *"#,
        )
        .bind(Uuid::new_v4())
        .bind(organization_slug)
        .bind(&stripe_customer_id)
        .fetch_optional(&mut *conn)
        .await?;

        match inserted {
            Some(customer) => {
                tracing::info!(
                    organization_slug = organization_slug,
                    stripe_customer_id = %customer.stripe_customer_id,
                    "created Stripe customer for organization"
                );
                Ok(customer)
            }
            None => {
                tracing::warn!(
                    organization_slug = organization_slug,
                    orphaned_stripe_customer_id = %stripe_customer_id,
                    "race detected in customer creation, cleaning up orphaned Stripe customer"
                );
                if let Err(e) = self.stripe.delete_customer(&stripe_customer_id).await {
                    tracing::warn!(
                        orphaned_stripe_customer_id = %stripe_customer_id,
                        error = %e,
                        "failed to delete orphaned Stripe customer"
                    );
                }
                BillingCustomer::query()
                    .select()
                    .r#where(
                        BillingCustomer::ORGANIZATION_SLUG,
                        "=",
                        organization_slug.to_owned(),
                    )
                    .first(&mut *conn)
                    .await?
                    .ok_or_else(|| BillingError::CustomerNotFound(organization_slug.to_owned()))
            }
        }
    }

    /// Finds the billing customer for an organization.
    pub async fn find_customer(
        &self,
        organization_slug: &str,
    ) -> Result<BillingCustomer, BillingError> {
        BillingCustomer::query()
            .select()
            .r#where(
                BillingCustomer::ORGANIZATION_SLUG,
                "=",
                organization_slug.to_owned(),
            )
            .first(&self.db)
            .await?
            .ok_or_else(|| BillingError::CustomerNotFound(organization_slug.to_owned()))
    }
}
