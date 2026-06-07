-- Create the kubernetes schema and the admin-managed cluster registry.
-- Each row stores an envelope-encrypted kubeconfig (see the frn-crypto crate):
-- nothing in this table is sensitive as long as the KEK
-- (KUBECONFIG_ENCRYPTION_KEY) is kept out of the database.
-- atlas:nolint

CREATE SCHEMA IF NOT EXISTS kubernetes;

CREATE TYPE kubernetes_cluster_health_status AS ENUM (
    'healthy',
    'unreachable'
);

CREATE TABLE kubernetes.cluster (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(63) NOT NULL UNIQUE
        CHECK (name ~ '^[a-z0-9]([a-z0-9-]*[a-z0-9])?$'),
    description TEXT,

    -- Envelope-encrypted kubeconfig material.
    encrypted_kubeconfig BYTEA NOT NULL,
    kubeconfig_nonce BYTEA NOT NULL,
    dek_encrypted BYTEA NOT NULL,
    dek_nonce BYTEA NOT NULL,
    key_version INTEGER NOT NULL DEFAULT 1,
    encryption_algorithm TEXT NOT NULL DEFAULT 'xchacha20poly1305'
        CHECK (encryption_algorithm IN ('xchacha20poly1305')),

    -- Non-sensitive metadata, displayed in the console and usable for dedup.
    api_server_url TEXT NOT NULL UNIQUE,
    ca_fingerprint TEXT,

    -- Version information reported by the API server during the last successful
    -- health check (GET /version). Nullable because the first health check may
    -- not have run yet or could have failed before returning version data.
    kubernetes_version TEXT,
    platform TEXT,

    -- Result of the most recent reachability check performed by the control
    -- plane. Defaults to 'unreachable' so a row that has never passed a check is
    -- never treated as healthy (create_cluster sets 'healthy' explicitly once
    -- the synchronous reachability check succeeds).
    health_status kubernetes_cluster_health_status NOT NULL DEFAULT 'unreachable',
    last_health_check_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Speeds up the key-rotation re-encryption job that scans rows by KEK version.
CREATE INDEX idx_kubernetes_cluster_key_version ON kubernetes.cluster (key_version);

-- Deduplicates clusters by CA fingerprint when one is recorded. NULLs (the
-- common case until a fingerprint is captured) are exempt, so the constraint
-- only blocks two clusters that genuinely present the same CA.
CREATE UNIQUE INDEX idx_kubernetes_cluster_ca_fingerprint
    ON kubernetes.cluster (ca_fingerprint)
    WHERE ca_fingerprint IS NOT NULL;
