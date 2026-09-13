-- atlas:nolint
-- FRA-15: per-seat pricing (per_unit + tiered).
--
-- Adds the product-level pricing model to managed.service_plan and the declared
-- seat count to billing.subscription. Both are backward compatible: existing
-- plans default to 'flat' and existing subscriptions keep a NULL seat count
-- (flat plans are billed with quantity 1).

-- ============================================================
-- managed.service_plan.pricing_model
-- flat      -> single fixed fee, quantity 1 (default; existing behaviour).
-- per_unit  -> fixed amount per seat, times the declared seat count.
-- tiered    -> graduated/volume tiers over the declared seat count.
-- ============================================================

ALTER TABLE managed.service_plan
    ADD COLUMN pricing_model VARCHAR(20) NOT NULL DEFAULT 'flat'
        CHECK (pricing_model IN ('flat', 'per_unit', 'tiered'));

-- ============================================================
-- billing.subscription.seats
-- Declared seat count for a per-seat plan, chosen at checkout and frozen for the
-- subscription's lifetime. NULL for flat plans; required (>= 1) at the
-- application layer when the plan's pricing_model is not 'flat'.
-- ============================================================

ALTER TABLE billing.subscription
    ADD COLUMN seats INTEGER
        CHECK (seats IS NULL OR seats >= 1);
