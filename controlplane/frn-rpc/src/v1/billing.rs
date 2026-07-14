//! gRPC service implementation for billing and subscription management.

use sqlx::{Pool, Postgres};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::error::Error;
use crate::timestamp::to_timestamp;
use frn_core::authorization::Authorize;
use frn_core::billing::{Billing, BillingError, BillingPeriod, BillingSubscription, StripeClient};
use frn_core::identity::IAM;

tonic::include_proto!("francenuage.fr.v1.billing");

pub struct BillingRpc<A: Authorize, S: StripeClient> {
    iam: IAM,
    billing: Billing<A, S>,
    pool: Pool<Postgres>,
}

impl<A: Authorize, S: StripeClient + 'static> BillingRpc<A, S> {
    pub fn new(iam: IAM, billing: Billing<A, S>, pool: Pool<Postgres>) -> Self {
        Self { iam, billing, pool }
    }
}

impl From<&BillingSubscription> for BillingSubscriptionProto {
    fn from(sub: &BillingSubscription) -> Self {
        Self {
            id: sub.id.to_string(),
            customer_id: sub.customer_id.to_string(),
            stripe_subscription_id: sub.stripe_subscription_id.clone(),
            plan_id: sub.plan_id.to_string(),
            instance_id: sub.instance_id.map(|id| id.to_string()),
            status: sub.status.to_string(),
            billing_period: sub.billing_period.to_string(),
            current_period_start: sub.current_period_start.map(to_timestamp),
            current_period_end: sub.current_period_end.map(to_timestamp),
            canceled_at: sub.canceled_at.map(to_timestamp),
            created_at: Some(to_timestamp(sub.created_at)),
        }
    }
}

fn billing_error_to_status(err: BillingError) -> Status {
    let message = err.to_string();
    match err {
        BillingError::CustomerNotFound(_)
        | BillingError::SubscriptionNotFound(_)
        | BillingError::PendingParamsNotFound(_) => Status::not_found(message),
        BillingError::InvalidStatusTransition { .. }
        | BillingError::PlanRequiresNoPayment(_)
        | BillingError::MissingStripePrice { .. } => Status::failed_precondition(message),
        BillingError::DuplicateEvent(_) => Status::already_exists(message),
        BillingError::InvalidWebhookSignature => Status::unauthenticated(message),
        BillingError::ManagedService(ref inner) => {
            use frn_core::managed::ManagedServiceError;
            match inner {
                ManagedServiceError::Authorization(_) => Status::permission_denied(message),
                ManagedServiceError::ServiceNotFound(_)
                | ManagedServiceError::VersionNotFound(_)
                | ManagedServiceError::PlanNotFound(_) => Status::not_found(message),
                ManagedServiceError::NoClusterMatchingDeployTarget(_)
                | ManagedServiceError::MissingDeployTarget(_) => {
                    Status::failed_precondition(message)
                }
                ManagedServiceError::PlanServiceMismatch { .. } => {
                    Status::failed_precondition(message)
                }
                ManagedServiceError::PlanNotActive(_)
                | ManagedServiceError::PlanRequiresPayment(_) => {
                    Status::failed_precondition(message)
                }
                _ => {
                    tracing::error!(error = %message, "internal billing error");
                    Status::internal("internal error")
                }
            }
        }
        _ => {
            tracing::error!(error = %message, "internal billing error");
            Status::internal("internal error")
        }
    }
}

fn parse_billing_period(s: &str) -> Result<BillingPeriod, Status> {
    match s {
        "monthly" => Ok(BillingPeriod::Monthly),
        "yearly" => Ok(BillingPeriod::Yearly),
        _ => Err(Status::invalid_argument(format!(
            "invalid billing_period: '{}', expected 'monthly' or 'yearly'",
            s
        ))),
    }
}

#[tonic::async_trait]
impl<A: Authorize + 'static, S: StripeClient + 'static> billing_service_server::BillingService
    for BillingRpc<A, S>
{
    async fn create_checkout_session(
        &self,
        request: Request<CreateCheckoutSessionRequest>,
    ) -> Result<Response<CreateCheckoutSessionResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();

        let version_id = req
            .version_id
            .parse::<Uuid>()
            .map_err(|_| Error::InvalidInput("invalid version_id".to_owned()))?;
        let plan_id = req
            .plan_id
            .parse::<Uuid>()
            .map_err(|_| Error::InvalidInput("invalid plan_id".to_owned()))?;
        let billing_period = parse_billing_period(&req.billing_period)?;

        let user_values = req
            .user_values
            .as_ref()
            .map(|s| serde_json::from_str(s))
            .transpose()
            .map_err(|e| Error::InvalidInput(format!("invalid user_values: {e}")))?;
        let secret_values = req
            .secret_values
            .as_ref()
            .map(|s| serde_json::from_str(s))
            .transpose()
            .map_err(|e| Error::InvalidInput(format!("invalid secret_values: {e}")))?;

        let mut tx = self.pool.begin().await.map_err(Error::from)?;

        let checkout_request = frn_core::billing::CreateCheckoutRequest {
            project_slug: req.project_slug,
            organization_slug: req.organization_slug,
            service_slug: req.service_slug,
            version_id,
            plan_id,
            billing_period,
            user_values,
            secret_values,
        };

        let response = self
            .billing
            .create_checkout_session(&principal, &mut tx, checkout_request)
            .await
            .map_err(billing_error_to_status)?;

        tx.commit().await.map_err(Error::from)?;

        Ok(Response::new(CreateCheckoutSessionResponse {
            subscription_id: response.subscription_id.to_string(),
            checkout_url: response.checkout_url,
        }))
    }

    async fn get_subscription(
        &self,
        request: Request<GetSubscriptionRequest>,
    ) -> Result<Response<GetSubscriptionResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();

        let subscription_id = req
            .subscription_id
            .parse::<Uuid>()
            .map_err(|_| Error::InvalidInput("invalid subscription_id".to_owned()))?;

        let subscription = self
            .billing
            .get_subscription_checked(&principal, subscription_id)
            .await
            .map_err(billing_error_to_status)?;

        Ok(Response::new(GetSubscriptionResponse {
            subscription: Some((&subscription).into()),
        }))
    }

    async fn list_subscriptions(
        &self,
        request: Request<ListSubscriptionsRequest>,
    ) -> Result<Response<ListSubscriptionsResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();

        let subscriptions = self
            .billing
            .list_subscriptions_checked(&principal, &req.organization_slug)
            .await
            .map_err(billing_error_to_status)?;

        Ok(Response::new(ListSubscriptionsResponse {
            subscriptions: subscriptions.iter().map(Into::into).collect(),
        }))
    }

    async fn cancel_subscription(
        &self,
        request: Request<CancelSubscriptionRequest>,
    ) -> Result<Response<CancelSubscriptionResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let req = request.into_inner();

        let subscription_id = req
            .subscription_id
            .parse::<Uuid>()
            .map_err(|_| Error::InvalidInput("invalid subscription_id".to_owned()))?;

        self.billing
            .cancel_subscription_checked(&principal, subscription_id)
            .await
            .map_err(billing_error_to_status)?;

        Ok(Response::new(CancelSubscriptionResponse {}))
    }
}
