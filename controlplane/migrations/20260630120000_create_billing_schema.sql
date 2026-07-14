-- atlas:nolint
-- Create the billing schema with tables for Stripe integration:
-- billing.customer, billing.subscription, billing.processed_event.
-- Also add Stripe price IDs and requires_payment flag to managed.service_plan.

-- ============================================================
-- SCHEMA
-- ============================================================

CREATE SCHEMA IF NOT EXISTS billing;

-- ============================================================
-- billing.customer
-- One Stripe customer per organization.
-- ============================================================

CREATE TABLE billing.customer (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_slug   CITEXT NOT NULL UNIQUE
                        REFERENCES organizations(slug) ON UPDATE CASCADE,
    stripe_customer_id  VARCHAR(255) NOT NULL UNIQUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_billing_customer_organization
    ON billing.customer(organization_slug);

CREATE INDEX idx_billing_customer_stripe
    ON billing.customer(stripe_customer_id);

-- ============================================================
-- billing.subscription
-- Tracks Stripe subscriptions tied to managed service instances.
-- ============================================================

CREATE TABLE billing.subscription (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id                 UUID NOT NULL
                                REFERENCES billing.customer(id),
    stripe_subscription_id      VARCHAR(255) UNIQUE,
    stripe_checkout_session_id  VARCHAR(255) UNIQUE,
    plan_id                     UUID NOT NULL
                                REFERENCES managed.service_plan(id),
    instance_id                 UUID
                                REFERENCES managed.service_instance(id),
    status                      VARCHAR(50) NOT NULL DEFAULT 'pending_payment'
                                CHECK (status IN (
                                    'pending_payment',
                                    'active',
                                    'past_due',
                                    'canceled',
                                    'incomplete'
                                )),
    billing_period              VARCHAR(10) NOT NULL
                                CHECK (billing_period IN ('monthly', 'yearly')),
    current_period_start        TIMESTAMPTZ,
    current_period_end          TIMESTAMPTZ,
    canceled_at                 TIMESTAMPTZ,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_billing_subscription_customer
    ON billing.subscription(customer_id);

CREATE INDEX idx_billing_subscription_stripe_sub
    ON billing.subscription(stripe_subscription_id);

CREATE INDEX idx_billing_subscription_stripe_session
    ON billing.subscription(stripe_checkout_session_id);

CREATE INDEX idx_billing_subscription_instance
    ON billing.subscription(instance_id);

CREATE INDEX idx_billing_subscription_plan
    ON billing.subscription(plan_id);

-- ============================================================
-- billing.processed_event
-- Webhook idempotency: stores processed Stripe event IDs.
-- ============================================================

CREATE TABLE billing.processed_event (
    event_id    VARCHAR(255) PRIMARY KEY,
    event_type  VARCHAR(100) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- Extend managed.service_plan with Stripe price IDs
-- ============================================================

ALTER TABLE managed.service_plan
    ADD COLUMN stripe_price_id_monthly VARCHAR(255),
    ADD COLUMN stripe_price_id_yearly  VARCHAR(255),
    ADD COLUMN requires_payment        BOOLEAN NOT NULL DEFAULT TRUE;

-- ============================================================
-- Pending instance params: stores create_instance parameters
-- between checkout creation and webhook confirmation.
-- Encrypted user_values/secret_values stored as opaque bytes.
-- ============================================================

CREATE TABLE billing.pending_instance_params (
    subscription_id     UUID PRIMARY KEY
                        REFERENCES billing.subscription(id) ON DELETE CASCADE,
    service_slug        VARCHAR(255) NOT NULL,
    version_id          UUID NOT NULL,
    project_slug        CITEXT NOT NULL,
    organization_slug   CITEXT NOT NULL,
    user_values         JSONB,
    secret_values       BYTEA,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
