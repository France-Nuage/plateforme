-- Create lib_fsm schema and workflow engine tables
-- atlas:nolint

-- lib_fsm: finite state machine library
-- Source: https://github.com/FGRibreau/lib_fsm/

CREATE SCHEMA IF NOT EXISTS lib_fsm;

-- type domains
CREATE DOMAIN lib_fsm.event_identifier AS varchar(30);
CREATE DOMAIN lib_fsm.abstract_state_identifier AS varchar(50) NOT NULL CHECK (length(value) >= 3);

-- abstract_state_machine
CREATE TABLE lib_fsm.abstract_state_machine (
    abstract_machine__id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(30) NOT NULL CHECK (length(name) >= 4),
    description text,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE FUNCTION lib_fsm.abstract_machine_create(
    name$ varchar(30),
    description$ text DEFAULT NULL,
    abstract_machine__id$ uuid DEFAULT gen_random_uuid(),
    created_at$ timestamptz DEFAULT now()
) RETURNS uuid AS $$
DECLARE id uuid;
BEGIN
    INSERT INTO lib_fsm.abstract_state_machine (abstract_machine__id, name, description, created_at)
    VALUES (abstract_machine__id$, name$, description$, created_at$)
    RETURNING abstract_machine__id INTO id;
    RETURN id;
END;
$$ LANGUAGE plpgsql;

-- abstract_state
CREATE TABLE lib_fsm.abstract_state (
    abstract_machine__id uuid NOT NULL REFERENCES lib_fsm.abstract_state_machine(abstract_machine__id) ON DELETE CASCADE ON UPDATE CASCADE,
    abstract_state__id uuid NOT NULL DEFAULT gen_random_uuid(),
    name lib_fsm.abstract_state_identifier NOT NULL,
    description text,
    is_initial boolean NOT NULL DEFAULT false,
    PRIMARY KEY (abstract_machine__id, abstract_state__id),
    UNIQUE (abstract_state__id)
);

CREATE UNIQUE INDEX abstract_state_abstract_machine_id_is_initial
    ON lib_fsm.abstract_state (abstract_machine__id, is_initial)
    WHERE is_initial = true;

CREATE FUNCTION lib_fsm.ensure_at_least_one_initial_state_per_abstract_machine() RETURNS trigger AS $$
DECLARE count int;
BEGIN
    IF new.is_initial = true THEN
        RETURN new;
    END IF;

    SELECT count(1) INTO count
    FROM lib_fsm.abstract_state as1
    WHERE as1.abstract_machine__id = new.abstract_machine__id
      AND as1.is_initial = true
    LIMIT 1;

    IF count = 0 THEN
        RAISE 'a state machine must have one initial state. none found.' USING errcode = 'check_violation';
    END IF;

    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER ensure_at_least_one_initial_state_per_abstract_machine
    BEFORE INSERT OR UPDATE ON lib_fsm.abstract_state
    FOR EACH ROW EXECUTE PROCEDURE lib_fsm.ensure_at_least_one_initial_state_per_abstract_machine();

CREATE FUNCTION lib_fsm.abstract_state_create(
    abstract_machine__id$ uuid,
    name$ lib_fsm.abstract_state_identifier,
    description$ text DEFAULT NULL,
    is_initial$ boolean DEFAULT false,
    abstract_state__id$ uuid DEFAULT gen_random_uuid()
) RETURNS uuid AS $$
DECLARE id uuid;
BEGIN
    INSERT INTO lib_fsm.abstract_state (abstract_state__id, abstract_machine__id, name, description, is_initial)
    VALUES (abstract_state__id$, abstract_machine__id$, name$, description$, is_initial$)
    RETURNING abstract_state__id INTO id;
    RETURN id;
END;
$$ LANGUAGE plpgsql;

-- abstract_transition
CREATE TABLE lib_fsm.abstract_transition (
    from_abstract_state__id uuid NOT NULL,
    to_abstract_state__id uuid NOT NULL,
    event lib_fsm.event_identifier NOT NULL,
    description text,
    created_at timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY (from_abstract_state__id) REFERENCES lib_fsm.abstract_state(abstract_state__id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (to_abstract_state__id) REFERENCES lib_fsm.abstract_state(abstract_state__id) ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE (from_abstract_state__id, event),
    PRIMARY KEY (from_abstract_state__id, event, to_abstract_state__id)
);

CREATE FUNCTION lib_fsm.ensure_from_and_to_state_have_same_machine__id() RETURNS trigger AS $$
DECLARE count int;
BEGIN
    SELECT count(1) INTO count
    FROM lib_fsm.abstract_state as1
    INNER JOIN lib_fsm.abstract_state as2 ON as2.abstract_state__id = new.to_abstract_state__id
    WHERE as1.abstract_state__id = new.from_abstract_state__id
      AND as1.abstract_machine__id = as2.abstract_machine__id
    LIMIT 1;

    IF count != 1 THEN
        RAISE 'both from and to state must have the same machine_id' USING errcode = 'check_violation';
    END IF;

    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER ensure_from_and_to_state_have_same_machine__id
    BEFORE INSERT OR UPDATE ON lib_fsm.abstract_transition
    FOR EACH ROW EXECUTE PROCEDURE lib_fsm.ensure_from_and_to_state_have_same_machine__id();

CREATE FUNCTION lib_fsm.abstract_transition_create(
    from_abstract_state__id$ uuid,
    event$ lib_fsm.event_identifier,
    to_abstract_state__id$ uuid,
    description$ text DEFAULT NULL,
    created_at$ timestamptz DEFAULT now()
) RETURNS void AS $$
BEGIN
    INSERT INTO lib_fsm.abstract_transition (from_abstract_state__id, to_abstract_state__id, event, description, created_at)
    VALUES (from_abstract_state__id$, to_abstract_state__id$, event$, description$, created_at$);
END;
$$ LANGUAGE plpgsql;

-- state_machine
CREATE TABLE lib_fsm.state_machine (
    state_machine__id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    abstract_state__id uuid NOT NULL,
    FOREIGN KEY (abstract_state__id) REFERENCES lib_fsm.abstract_state(abstract_state__id) ON DELETE CASCADE ON UPDATE CASCADE
);

-- state_machine_event (transition history)
CREATE TABLE lib_fsm.state_machine_event (
    event_id serial PRIMARY KEY,
    state_machine__id uuid NOT NULL REFERENCES lib_fsm.state_machine(state_machine__id) ON DELETE CASCADE ON UPDATE CASCADE,
    abstract_state__id uuid NOT NULL REFERENCES lib_fsm.abstract_state(abstract_state__id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    event varchar(30),
    abstract_state_name lib_fsm.abstract_state_identifier NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- views
CREATE OR REPLACE VIEW lib_fsm.abstract_state_machine_transitions AS
SELECT
    ms_previous.abstract_machine__id,
    ms_previous.abstract_state__id AS from_abstract_state__id,
    ms_previous.name AS from_state_name,
    ms_previous.description AS from_state_description,
    mt.event,
    mt.description,
    ms_next.abstract_state__id AS to_abstract_state__id,
    ms_next.name AS to_state_name,
    ms_next.description AS to_state_description
FROM lib_fsm.abstract_transition mt
INNER JOIN lib_fsm.abstract_state ms_previous ON mt.from_abstract_state__id = ms_previous.abstract_state__id
INNER JOIN lib_fsm.abstract_state ms_next ON mt.to_abstract_state__id = ms_next.abstract_state__id;

CREATE OR REPLACE VIEW lib_fsm.state_machine_events AS
SELECT
    sme.event_id,
    sme.state_machine__id,
    sme.created_at,
    sme.abstract_state_name AS state_name,
    ast.description AS state_description,
    sme.event AS event_name
FROM lib_fsm.state_machine_event sme
LEFT OUTER JOIN lib_fsm.abstract_state ast ON ast.abstract_state__id = sme.abstract_state__id;

-- internal helper: store event in history
CREATE FUNCTION lib_fsm._state_machine_store_event(
    abstract_state__id$ uuid,
    state_machine__id$ uuid,
    event$ lib_fsm.event_identifier
) RETURNS void AS $$
DECLARE name$ varchar(30);
BEGIN
    SELECT name INTO name$ FROM lib_fsm.abstract_state WHERE abstract_state__id = abstract_state__id$;
    INSERT INTO lib_fsm.state_machine_event (state_machine__id, abstract_state__id, event, abstract_state_name, created_at)
    VALUES (state_machine__id$, abstract_state__id$, event$, name$, DEFAULT);
END;
$$ LANGUAGE plpgsql;

-- state_machine_create
CREATE FUNCTION lib_fsm.state_machine_create(
    abstract_state_machine__id_or_abstract_state__id$ uuid,
    state_machine__id$ uuid DEFAULT gen_random_uuid()
) RETURNS uuid AS $$
DECLARE initial_abstract_state__id$ uuid;
BEGIN
    SELECT abstract_state__id INTO initial_abstract_state__id$
    FROM lib_fsm.abstract_state abst
    WHERE abst.abstract_machine__id = abstract_state_machine__id_or_abstract_state__id$
      AND is_initial = true;

    IF NOT FOUND THEN
        initial_abstract_state__id$ = abstract_state_machine__id_or_abstract_state__id$;
    END IF;

    INSERT INTO lib_fsm.state_machine (state_machine__id, abstract_state__id)
    VALUES (state_machine__id$, initial_abstract_state__id$);

    PERFORM lib_fsm._state_machine_store_event(
        abstract_state__id$ => initial_abstract_state__id$,
        state_machine__id$ => state_machine__id$,
        event$ => NULL
    );

    RETURN state_machine__id$;
END;
$$ LANGUAGE plpgsql;

CREATE TYPE lib_fsm.state_machine_state AS (
    abstract_state__id uuid,
    name lib_fsm.abstract_state_identifier,
    description text,
    created_at timestamptz
);

-- state_machine_transition
CREATE FUNCTION lib_fsm.state_machine_transition(
    state_machine__id$ uuid,
    event$ lib_fsm.event_identifier,
    dry_run$ boolean DEFAULT false
) RETURNS lib_fsm.state_machine_state AS $$
DECLARE to_state lib_fsm.state_machine_state;
BEGIN
    SELECT ast.abstract_state__id, ast.name, ast.description, now()
    INTO to_state
    FROM lib_fsm.state_machine sm
    INNER JOIN lib_fsm.abstract_transition abtr ON abtr.from_abstract_state__id = sm.abstract_state__id
    INNER JOIN lib_fsm.abstract_state ast ON ast.abstract_state__id = abtr.to_abstract_state__id
    WHERE sm.state_machine__id = state_machine__id$
      AND abtr.event = event$;

    IF NOT FOUND THEN
        RAISE SQLSTATE 'P0001' USING message = 'Invalid event for this machine';
    END IF;

    IF dry_run$ = true THEN
        RETURN to_state;
    END IF;

    UPDATE lib_fsm.state_machine
    SET abstract_state__id = to_state.abstract_state__id
    WHERE state_machine.state_machine__id = state_machine__id$;

    PERFORM lib_fsm._state_machine_store_event(
        abstract_state__id$ => to_state.abstract_state__id,
        state_machine__id$ => state_machine__id$,
        event$ => event$
    );

    RETURN to_state;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION lib_fsm.state_machine_get(state_machine__id$ uuid) RETURNS lib_fsm.state_machine_state AS $$
DECLARE state lib_fsm.state_machine_state;
BEGIN
    SELECT abstract_state.abstract_state__id, abstract_state.name, abstract_state.description, events_subq.created_at
    INTO state
    FROM lib_fsm.state_machine
    INNER JOIN lib_fsm.abstract_state USING (abstract_state__id)
    INNER JOIN (
        SELECT fsm_event.state_machine__id, fsm_event.abstract_state__id, max(fsm_event.created_at) AS created_at
        FROM lib_fsm.state_machine_event AS fsm_event
        GROUP BY fsm_event.state_machine__id, fsm_event.abstract_state__id
    ) AS events_subq
    ON events_subq.state_machine__id = state_machine.state_machine__id
       AND events_subq.abstract_state__id = abstract_state.abstract_state__id
    WHERE state_machine.state_machine__id = state_machine__id$;

    IF NOT FOUND THEN
        RAISE SQLSTATE '42P01' USING message = 'state_machine__id not found', hint = state_machine__id$;
    END IF;

    RETURN state;
END;
$$ STABLE SECURITY DEFINER LANGUAGE plpgsql;

CREATE FUNCTION lib_fsm.state_machine_delete(state_machine__id$ uuid) RETURNS void AS $$
BEGIN
    DELETE FROM lib_fsm.state_machine WHERE state_machine__id = state_machine__id$;
END;
$$ LANGUAGE plpgsql;

-- END OF lib_fsm

-- Create workflow schema
CREATE SCHEMA IF NOT EXISTS workflow;

-- Seed the workflow_execution_status FSM
DO $$
DECLARE
    abstract_machine_id uuid := '0199f388-fc9f-7374-b6b1-896342a0d4d9';
    pending_state_id uuid;
    running_state_id uuid;
    will_retry_state_id uuid;
    completed_state_id uuid;
    failed_state_id uuid;
BEGIN
    PERFORM lib_fsm.abstract_machine_create('workflow_execution_status', 'State machine for workflow execution status', abstract_machine_id);

    pending_state_id    := lib_fsm.abstract_state_create(abstract_machine_id, 'pending',    'Workflow is pending execution', true);
    running_state_id    := lib_fsm.abstract_state_create(abstract_machine_id, 'running',    'Workflow is currently running');
    will_retry_state_id := lib_fsm.abstract_state_create(abstract_machine_id, 'will_retry', 'Workflow will be retried');
    completed_state_id  := lib_fsm.abstract_state_create(abstract_machine_id, 'completed',  'Workflow has completed successfully');
    failed_state_id     := lib_fsm.abstract_state_create(abstract_machine_id, 'failed',     'Workflow has failed');

    PERFORM lib_fsm.abstract_transition_create(pending_state_id,    'run',        running_state_id);
    PERFORM lib_fsm.abstract_transition_create(running_state_id,    'will_retry', will_retry_state_id);
    PERFORM lib_fsm.abstract_transition_create(running_state_id,    'complete',   completed_state_id);
    PERFORM lib_fsm.abstract_transition_create(running_state_id,    'fail',       failed_state_id);
    PERFORM lib_fsm.abstract_transition_create(will_retry_state_id, 'retry',      running_state_id);
END;
$$;

-- Create the workflow.execution table
CREATE TABLE workflow.execution (
    execution_id UUID PRIMARY KEY NOT NULL,
    initiated_by_user UUID,
    initiated_by_workflow UUID REFERENCES workflow.execution(execution_id) ON DELETE SET NULL,
    status UUID NOT NULL DEFAULT lib_fsm.state_machine_create('0199f388-fc9f-7374-b6b1-896342a0d4d9'),
    soft_try_count INTEGER NOT NULL DEFAULT 0,
    hard_try_count INTEGER NOT NULL DEFAULT 0,
    max_try_count INTEGER NOT NULL,
    definition JSONB NOT NULL,
    next_retry_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    locked_until TIMESTAMPTZ,
    idempotency_key TEXT,
    CONSTRAINT fk_status FOREIGN KEY (status) REFERENCES lib_fsm.state_machine(state_machine__id)
);

CREATE UNIQUE INDEX idx_workflow_execution_idempotency_key
    ON workflow.execution(idempotency_key)
    WHERE idempotency_key IS NOT NULL;

-- Create the workflow.execution_dependency table
CREATE TABLE workflow.execution_dependency (
    execution_id UUID NOT NULL REFERENCES workflow.execution(execution_id) ON DELETE CASCADE,
    dependency_id UUID NOT NULL REFERENCES workflow.execution(execution_id) ON DELETE CASCADE,
    PRIMARY KEY (execution_id, dependency_id)
);

-- Index for efficient workflow polling
CREATE INDEX idx_workflow_execution_polling
    ON workflow.execution(next_retry_at, locked_until);

-- Drop the legacy operations table, replaced by the workflow engine (workflow.execution)
DROP INDEX IF EXISTS idx_operations_status_created_at;
DROP INDEX IF EXISTS idx_operations_status;
DROP TABLE IF EXISTS operations;
