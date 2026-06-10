BEGIN;

INSERT INTO organizations (id, name, slug)
VALUES ('88f42823-2724-4d3f-a42b-659ba6aa2b6c', 'acme', 'acme')
ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name;

INSERT INTO users (id, email, is_admin)
VALUES ('01965d6b-0000-7000-8000-000000000001', 'wile.coyote@acme.org', true)
ON CONFLICT (email) DO UPDATE SET is_admin = EXCLUDED.is_admin;

INSERT INTO organization_user (organization_id, user_id)
VALUES ('88f42823-2724-4d3f-a42b-659ba6aa2b6c', '01965d6b-0000-7000-8000-000000000001')
ON CONFLICT (organization_id, user_id) DO NOTHING;

INSERT INTO projects (id, name, organization_id)
VALUES ('01965d6b-0000-7000-8000-000000000002', 'unattributed', '88f42823-2724-4d3f-a42b-659ba6aa2b6c')
ON CONFLICT (id) DO NOTHING;

COMMIT;

-- Hypervisor seeding (run manually with: psql -v url=... -v token=... -v storage=...)
-- BEGIN;
-- WITH zone AS (
--   INSERT INTO zones (name)
--   VALUES ('ACME Mesa Data Facility')
--   RETURNING id
-- )
-- INSERT INTO hypervisors (url, authorization_token, storage_name, organization_id, zone_id)
-- SELECT :url, :token, :storage, '88f42823-2724-4d3f-a42b-659ba6aa2b6c', zone.id
-- FROM zone;
-- COMMIT;

-- Seed managed services catalog
BEGIN;

-- Vaultwarden
WITH svc AS (
  INSERT INTO managed.service (slug, name, description, category, database_engine)
  VALUES (
    'vaultwarden',
    'Vaultwarden',
    'Gestionnaire de mots de passe open source compatible Bitwarden. Leger, rapide, ideal pour les equipes.',
    'security',
    'cnpg'
  )
  ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
  RETURNING id
),
tier_starter AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'starter', 'Starter', 'Pour les petites equipes', 999,
    '{"resources": {"requests": {"cpu": "100m", "memory": "256Mi"}, "limits": {"cpu": "500m", "memory": "512Mi"}}, "cnpg": {"instances": 1, "storage": {"size": "5Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
),
tier_pro AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'pro', 'Pro', 'Pour les equipes en croissance', 2999,
    '{"resources": {"requests": {"cpu": "250m", "memory": "512Mi"}, "limits": {"cpu": "1", "memory": "1Gi"}}, "cnpg": {"instances": 2, "storage": {"size": "10Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
),
tier_enterprise AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'enterprise', 'Enterprise', 'Haute disponibilite', 9999,
    '{"resources": {"requests": {"cpu": "500m", "memory": "1Gi"}, "limits": {"cpu": "2", "memory": "2Gi"}}, "cnpg": {"instances": 3, "storage": {"size": "20Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
)
INSERT INTO managed.service_version (service_id, chart_version, app_version, oci_reference)
SELECT id, '0.1.0', '1.35.4', 'oci://registry.gitlab.com/getbunker-france-nuage/france-nuage/charts/vaultwarden'
FROM svc
ON CONFLICT (service_id, chart_version) DO NOTHING;

-- Nextcloud
WITH svc AS (
  INSERT INTO managed.service (slug, name, description, category, database_engine)
  VALUES (
    'nextcloud',
    'Nextcloud',
    'Plateforme collaborative complète : fichiers, calendrier, contacts, visioconference. Alternative souveraine à Google Workspace.',
    'collaboration',
    'cnpg'
  )
  ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
  RETURNING id
),
tier_starter AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'starter', 'Starter', 'Pour un usage personnel ou petite equipe', 1999,
    '{"resources": {"requests": {"cpu": "250m", "memory": "512Mi"}, "limits": {"cpu": "1", "memory": "1Gi"}}, "cnpg": {"instances": 1, "storage": {"size": "20Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
),
tier_pro AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'pro', 'Pro', 'Pour les equipes de taille moyenne', 4999,
    '{"resources": {"requests": {"cpu": "500m", "memory": "1Gi"}, "limits": {"cpu": "2", "memory": "2Gi"}}, "cnpg": {"instances": 2, "storage": {"size": "50Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
),
tier_enterprise AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'enterprise', 'Enterprise', 'Pour les grandes organisations', 14999,
    '{"resources": {"requests": {"cpu": "1", "memory": "2Gi"}, "limits": {"cpu": "4", "memory": "4Gi"}}, "cnpg": {"instances": 3, "storage": {"size": "100Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
)
INSERT INTO managed.service_version (service_id, chart_version, app_version, oci_reference)
SELECT id, '1.0.0', '32.0.5', 'oci://registry.gitlab.com/getbunker-france-nuage/france-nuage/charts/nextcloud-stack'
FROM svc
ON CONFLICT (service_id, chart_version) DO NOTHING;

-- n8n
WITH svc AS (
  INSERT INTO managed.service (slug, name, description, category, database_engine)
  VALUES (
    'n8n',
    'n8n',
    'Plateforme d''automatisation de workflows. Connectez vos applications et automatisez vos processus metier.',
    'automation',
    'cnpg'
  )
  ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
  RETURNING id
),
tier_starter AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'starter', 'Starter', 'Automatisations simples', 1499,
    '{"resources": {"requests": {"cpu": "200m", "memory": "512Mi"}, "limits": {"cpu": "1", "memory": "1Gi"}}, "cnpg": {"instances": 1, "storage": {"size": "5Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
),
tier_pro AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'pro', 'Pro', 'Workflows complexes et haute frequence', 3999,
    '{"resources": {"requests": {"cpu": "500m", "memory": "1Gi"}, "limits": {"cpu": "2", "memory": "2Gi"}}, "cnpg": {"instances": 2, "storage": {"size": "10Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
),
tier_enterprise AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'enterprise', 'Enterprise', 'Volume illimite, SLA premium', 9999,
    '{"resources": {"requests": {"cpu": "1", "memory": "2Gi"}, "limits": {"cpu": "4", "memory": "4Gi"}}, "cnpg": {"instances": 2, "storage": {"size": "20Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
)
INSERT INTO managed.service_version (service_id, chart_version, app_version, oci_reference)
SELECT id, '0.1.0', '1.76.1', 'oci://registry.gitlab.com/getbunker-france-nuage/france-nuage/charts/n8n'
FROM svc
ON CONFLICT (service_id, chart_version) DO NOTHING;

-- Metabase
WITH svc AS (
  INSERT INTO managed.service (slug, name, description, category, database_engine)
  VALUES (
    'metabase',
    'Metabase',
    'Outil de Business Intelligence open source. Creez des dashboards et explorez vos donnees sans SQL.',
    'analytics',
    'cnpg'
  )
  ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
  RETURNING id
),
tier_starter AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'starter', 'Starter', 'Exploration de donnees basique', 1999,
    '{"resources": {"requests": {"cpu": "200m", "memory": "1Gi"}, "limits": {"cpu": "1", "memory": "2Gi"}}, "cnpg": {"instances": 1, "storage": {"size": "10Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
),
tier_pro AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'pro', 'Pro', 'Dashboards avances et partage equipe', 4999,
    '{"resources": {"requests": {"cpu": "500m", "memory": "2Gi"}, "limits": {"cpu": "2", "memory": "4Gi"}}, "cnpg": {"instances": 2, "storage": {"size": "20Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
),
tier_enterprise AS (
  INSERT INTO managed.service_tier (service_id, slug, name, description, price_cents, preset_values)
  SELECT id, 'enterprise', 'Enterprise', 'BI a grande echelle', 12999,
    '{"resources": {"requests": {"cpu": "1", "memory": "4Gi"}, "limits": {"cpu": "4", "memory": "8Gi"}}, "cnpg": {"instances": 2, "storage": {"size": "50Gi"}}}'::jsonb
  FROM svc
  ON CONFLICT (service_id, slug) DO NOTHING
)
INSERT INTO managed.service_version (service_id, chart_version, app_version, oci_reference)
SELECT id, '0.1.0', '0.54.3', 'oci://registry.gitlab.com/getbunker-france-nuage/france-nuage/charts/metabase'
FROM svc
ON CONFLICT (service_id, chart_version) DO NOTHING;

COMMIT;
