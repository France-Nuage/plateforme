-- Add the deploy_target column to managed.service.
--
-- Label selector resolved at instance deployment: a JSONB object of
-- key/value pairs (e.g. {"availability": "ft"}). Only clusters carrying
-- every pair are eligible to host instances of this service. NULL or {}
-- means the service declares no target and cannot be deployed (the
-- service layer rejects it with a typed error).
--
-- This column was originally added by editing the already-applied
-- 20260513120000_create_managed_services.sql migration, which Atlas never
-- replays on existing databases. This migration carries the change properly.
ALTER TABLE managed.service
    ADD COLUMN deploy_target JSONB
        CHECK (deploy_target IS NULL OR jsonb_typeof(deploy_target) = 'object');
