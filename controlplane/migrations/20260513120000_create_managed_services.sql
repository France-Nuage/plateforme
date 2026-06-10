-- Create managed services schema, enums, tables, and FSM
-- atlas:nolint

CREATE SCHEMA IF NOT EXISTS managed;

-- Enums
CREATE TYPE managed_service_category AS ENUM (
    'security',
    'collaboration',
    'analytics',
    'database',
    'automation',
    'cms',
    'erp',
    'storage',
    'dashboard'
);

CREATE TYPE managed_database_engine AS ENUM (
    'cnpg',
    'mariadb'
);

-- managed.service
CREATE TABLE managed.service (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(63) NOT NULL UNIQUE
        CHECK (slug ~ '^[a-z0-9]([a-z0-9-]*[a-z0-9])?$'),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    category managed_service_category NOT NULL,
    database_engine managed_database_engine,
    icon_url TEXT,
    -- Label selector resolved at instance deployment: a JSONB object of
    -- key/value pairs (e.g. {"availability": "ft"}). Only clusters carrying
    -- every pair are eligible to host instances of this service. NULL or {}
    -- means the service declares no target and cannot be deployed (the
    -- service layer rejects it with a typed error).
    deploy_target JSONB
        CHECK (deploy_target IS NULL OR jsonb_typeof(deploy_target) = 'object'),
    deactivated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- managed.service_version
CREATE TABLE managed.service_version (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_id UUID NOT NULL REFERENCES managed.service(id) ON DELETE CASCADE,
    chart_version VARCHAR(50) NOT NULL
        CHECK (chart_version ~ '^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$'),
    app_version VARCHAR(50),
    oci_reference TEXT NOT NULL
        CHECK (oci_reference LIKE 'oci://%'),
    configurable_values_schema JSONB,
    deactivated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (service_id, chart_version)
);

-- FSM: managed_service_instance_status
DO $$
DECLARE
    abstract_machine_id UUID := '019a0001-0000-7000-b000-000000000001';
    provisioning_id UUID;
    running_id UUID;
    upgrading_id UUID;
    failed_id UUID;
    deleting_id UUID;
    deleted_id UUID;
BEGIN
    PERFORM lib_fsm.abstract_machine_create(
        'managed_instance_status',
        'State machine for managed service instance lifecycle',
        abstract_machine_id
    );

    provisioning_id := lib_fsm.abstract_state_create(abstract_machine_id, 'provisioning', 'Instance is being provisioned', true);
    running_id      := lib_fsm.abstract_state_create(abstract_machine_id, 'running',      'Instance is running');
    upgrading_id    := lib_fsm.abstract_state_create(abstract_machine_id, 'upgrading',    'Instance is being upgraded');
    failed_id       := lib_fsm.abstract_state_create(abstract_machine_id, 'failed',       'Instance has failed');
    deleting_id     := lib_fsm.abstract_state_create(abstract_machine_id, 'deleting',     'Instance is being deleted');
    deleted_id      := lib_fsm.abstract_state_create(abstract_machine_id, 'deleted',      'Instance has been deleted');

    -- provisioning transitions
    PERFORM lib_fsm.abstract_transition_create(provisioning_id, 'provision_complete', running_id);
    PERFORM lib_fsm.abstract_transition_create(provisioning_id, 'fail',              failed_id);

    -- running transitions
    PERFORM lib_fsm.abstract_transition_create(running_id, 'upgrade', upgrading_id);
    PERFORM lib_fsm.abstract_transition_create(running_id, 'delete',  deleting_id);
    PERFORM lib_fsm.abstract_transition_create(running_id, 'fail',    failed_id);

    -- upgrading transitions
    PERFORM lib_fsm.abstract_transition_create(upgrading_id, 'upgrade_complete', running_id);
    PERFORM lib_fsm.abstract_transition_create(upgrading_id, 'fail',             failed_id);

    -- failed transitions
    PERFORM lib_fsm.abstract_transition_create(failed_id, 'retry',  provisioning_id);
    PERFORM lib_fsm.abstract_transition_create(failed_id, 'delete', deleting_id);

    -- deleting transitions
    PERFORM lib_fsm.abstract_transition_create(deleting_id, 'delete_complete', deleted_id);
    PERFORM lib_fsm.abstract_transition_create(deleting_id, 'fail',            failed_id);
END;
$$;

-- managed.service_instance
CREATE TABLE managed.service_instance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_id UUID NOT NULL REFERENCES managed.service(id),
    version_id UUID NOT NULL REFERENCES managed.service_version(id),
    project_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    namespace VARCHAR(253) NOT NULL
        CHECK (namespace ~ '^[a-z0-9]([a-z0-9.-]*[a-z0-9])?$'),
    release_name VARCHAR(53) NOT NULL
        CHECK (release_name ~ '^[a-z0-9]([a-z0-9-]*[a-z0-9])?$'),
    user_values JSONB,
    status UUID NOT NULL DEFAULT lib_fsm.state_machine_create('019a0001-0000-7000-b000-000000000001'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_status FOREIGN KEY (status) REFERENCES lib_fsm.state_machine(state_machine__id),
    UNIQUE (namespace),
    UNIQUE (release_name)
);

CREATE INDEX idx_managed_service_instance_project ON managed.service_instance(project_id);
CREATE INDEX idx_managed_service_instance_organization ON managed.service_instance(organization_id);
CREATE INDEX idx_managed_service_instance_service ON managed.service_instance(service_id);
