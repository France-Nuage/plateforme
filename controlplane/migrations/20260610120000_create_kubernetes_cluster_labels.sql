-- atlas:nolint
-- Cluster labels and their attachment table.
--
-- Labels are key/value pairs (e.g. availability=ft, region=fr) attached to
-- clusters by platform admins. Managed services declare the labels they
-- require through managed.service.deploy_target; at instance deployment the
-- control plane picks a healthy cluster carrying ALL the required pairs.
--
-- CITEXT makes key and value case-insensitive: availability=FT and
-- availability=ft are the same label, so a deploy_target can never miss a
-- cluster because of casing.

CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE kubernetes.label (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key CITEXT NOT NULL
        CHECK (length(key) < 50 AND key ~ '^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$'),
    value CITEXT NOT NULL
        CHECK (length(value) < 50 AND value ~ '^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$'),

    -- TRUE marks a label owned by the control plane (written by internal
    -- code/seed only). The API refuses create/delete/attach/detach on system
    -- labels, even for platform admins.
    system BOOLEAN NOT NULL DEFAULT false,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (key, value)
);

CREATE TABLE kubernetes.cluster_label (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cluster_id UUID NOT NULL REFERENCES kubernetes.cluster (id) ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES kubernetes.label (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (cluster_id, label_id)
);

-- Speeds up "which clusters carry this label" lookups performed by the
-- deploy-target matching query (the UNIQUE above already covers the
-- cluster_id-first direction).
CREATE INDEX idx_kubernetes_cluster_label_label
    ON kubernetes.cluster_label (label_id);
