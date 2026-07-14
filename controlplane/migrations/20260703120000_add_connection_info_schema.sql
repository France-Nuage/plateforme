ALTER TABLE managed.service_version
    ADD COLUMN IF NOT EXISTS connection_info_schema JSONB;

COMMENT ON COLUMN managed.service_version.connection_info_schema IS
    'JSON schema describing how to fetch and display connection info for deployed instances. '
    'Defines which K8s secrets to read and how to render each field in the console.';
