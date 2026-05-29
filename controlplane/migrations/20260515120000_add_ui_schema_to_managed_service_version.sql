-- Adds the rjsf UI Schema to managed.service_version.
-- The chart CI publishes both the validation schema (configurable_values_schema)
-- and the rendering schema (ui_schema, rjsf-compatible) so the console can render
-- a polished form without inferring layout from JSON Schema alone.
ALTER TABLE managed.service_version
    ADD COLUMN ui_schema JSONB;
