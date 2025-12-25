-- Create "operations" table for async operation tracking and external system synchronization
CREATE TABLE "public"."operations" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "name" text NOT NULL UNIQUE,

    -- Operation metadata
    "operation_type" text NOT NULL,
    "target_backend" text NOT NULL,
    "resource_type" text NOT NULL,
    "resource_id" uuid NOT NULL,

    -- State machine: PENDING, RUNNING, SUCCEEDED, FAILED, CANCELLED
    "status" text NOT NULL DEFAULT 'PENDING',

    -- Payload
    "input" jsonb NOT NULL DEFAULT '{}',
    "output" jsonb,
    "error_code" text,
    "error_message" text,

    -- Retry tracking
    "attempt_count" int NOT NULL DEFAULT 0,
    "max_attempts" int NOT NULL DEFAULT 5,
    "next_retry_at" timestamptz,
    "last_error" text,

    -- Idempotency
    "idempotency_key" text UNIQUE,

    -- Timestamps
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "started_at" timestamptz,
    "completed_at" timestamptz,
    "updated_at" timestamptz NOT NULL DEFAULT now(),

    PRIMARY KEY ("id")
);

-- Index for efficient status-based polling
CREATE INDEX idx_operations_status ON operations(status);

-- Index for backend-specific queries
CREATE INDEX idx_operations_backend ON operations(target_backend, status);

-- Index for resource-specific queries
CREATE INDEX idx_operations_resource ON operations(resource_type, resource_id);

-- Index for retry scheduling (only for pending/running operations)
CREATE INDEX idx_operations_next_retry ON operations(next_retry_at)
    WHERE status = 'PENDING' OR status = 'RUNNING';

-- Composite index for the polling query pattern
CREATE INDEX idx_operations_polling ON operations(status, next_retry_at, created_at)
    WHERE status = 'PENDING' OR status = 'RUNNING';
