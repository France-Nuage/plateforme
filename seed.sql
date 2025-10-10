BEGIN;

WITH organization AS (
  INSERT INTO organizations (name)
  VALUES ('ACME')
  RETURNING id
),
zone AS (
  INSERT INTO zones (name)
  VALUES ('ACME Mesa Data Facility')
  RETURNING id
)
INSERT INTO hypervisors (
  url,
  authorization_token,
  storage_name,
  organization_id,
  zone_id
)
SELECT
  :url,
  :token,
  :storage,
  organization.id,
  zone.id
FROM organization, zone;

COMMIT;
