-- Add managed service plans (pricing tiers) and link instances to plans.
-- atlas:nolint

-- managed.service_plan
CREATE TABLE managed.service_plan (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_id          UUID NOT NULL REFERENCES managed.service(id),
    slug                VARCHAR(100) NOT NULL
                        CHECK (slug ~ '^[a-z0-9]([a-z0-9-]*[a-z0-9])?$'),
    name                VARCHAR(100) NOT NULL,
    description         TEXT,
    status              TEXT NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active', 'archived')),
    highlighted         BOOLEAN NOT NULL DEFAULT FALSE,
    values_override     JSONB,
    entitlements        JSONB NOT NULL DEFAULT '[]'::JSONB,
    price_monthly_cents BIGINT,
    price_yearly_cents  BIGINT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (service_id, slug)
);

CREATE INDEX idx_managed_service_plan_service
    ON managed.service_plan(service_id);

-- Add nullable plan_id to service_instance (backwards-compatible with
-- existing instances; the application layer enforces non-null for new ones).
ALTER TABLE managed.service_instance
    ADD COLUMN plan_id UUID REFERENCES managed.service_plan(id);

CREATE INDEX idx_managed_service_instance_plan
    ON managed.service_instance(plan_id);

-- Rebuild the view to expose plan_id.
DROP VIEW managed.service_instance_view;

CREATE VIEW managed.service_instance_view AS
SELECT si.id,
       si.service_id,
       si.version_id,
       si.plan_id,
       si.project_id,
       si.organization_id,
       si.cluster_id,
       si.namespace,
       si.release_name,
       si.user_values,
       abs.name AS status,
       si.created_at
FROM managed.service_instance si
JOIN lib_fsm.state_machine sm ON sm.state_machine__id = si.status
JOIN lib_fsm.abstract_state abs ON abs.abstract_state__id = sm.abstract_state__id;
