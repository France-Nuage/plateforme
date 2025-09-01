BEGIN;

WITH organization AS (
  INSERT INTO organizations (name)
  VALUES ('ACME')
  RETURNING id
),
datacenter AS (
  INSERT INTO datacenters (name)
  VALUES ('ACME Mesa Data Facility')
  RETURNING id
)
INSERT INTO hypervisors (
  url,
  authorization_token,
  storage_name,
  organization_id,
  datacenter_id
)
SELECT
  :url,
  :token,
  :storage,
  organization.id,
  datacenter.id
FROM organization, datacenter;

COMMIT;
