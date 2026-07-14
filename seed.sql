BEGIN;

INSERT INTO organizations (slug, name)
VALUES ('acme', 'acme')
ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name;

INSERT INTO users (id, email, is_admin)
VALUES ('01965d6b-0000-7000-8000-000000000001', 'wile.coyote@acme.org', true)
ON CONFLICT (email) DO UPDATE SET is_admin = EXCLUDED.is_admin;

INSERT INTO organization_user (organization_slug, user_id)
VALUES ('acme', '01965d6b-0000-7000-8000-000000000001')
ON CONFLICT (user_id, organization_slug) DO NOTHING;

INSERT INTO projects (slug, name, organization_slug)
VALUES ('unattributed', 'unattributed', 'acme')
ON CONFLICT (slug) DO NOTHING;

COMMIT;

-- Seed managed services catalog (mirrors controlplane/seed/managed/postgres.yaml)
BEGIN;

WITH svc AS (
  INSERT INTO managed.service (slug, name, description, category, database_engine, deploy_target)
  VALUES (
    'postgres',
    'PostgreSQL',
    'Base de données PostgreSQL managée via CloudNativePG. Haute disponibilité, sauvegardes automatiques, monitoring intégré.',
    'database',
    'cnpg',
    '{"availability": "fr"}'::jsonb
  )
  ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    category = EXCLUDED.category,
    database_engine = EXCLUDED.database_engine,
    deploy_target = EXCLUDED.deploy_target
  RETURNING id
),
plan_standard AS (
  INSERT INTO managed.service_plan (service_id, slug, name, description, status, highlighted, requires_payment, values_override, entitlements, price_monthly_cents, price_yearly_cents, stripe_price_id_monthly, stripe_price_id_yearly)
  SELECT id, 'cnpg-standard', 'PostgreSQL Standard',
    'Instance single-replica, sauvegarde quotidienne, support e-mail.',
    'active', false, true,
    '{"postgres": {"replicas": 1, "size": "10Gi", "resources": {"requests": {"cpu": "100m", "memory": "256Mi"}, "limits": {"cpu": "500m", "memory": "512Mi"}}}}'::jsonb,
    '[{"key": "support_level", "label": "Support", "value": "Email"}, {"key": "gti", "label": "Temps d''intervention garanti", "value": "24h"}, {"key": "backup", "label": "Sauvegarde", "value": "Quotidienne"}, {"key": "storage", "label": "Stockage", "value": "10 Go"}]'::jsonb,
    1499, 16189,
    'price_1Tp1KICMRVnojgCRFEmZuhqw', 'price_1Tp1KOCMRVnojgCRpTV4bd17'
  FROM svc
  ON CONFLICT (service_id, slug) DO UPDATE SET
    name = EXCLUDED.name, description = EXCLUDED.description, requires_payment = EXCLUDED.requires_payment,
    price_monthly_cents = EXCLUDED.price_monthly_cents, price_yearly_cents = EXCLUDED.price_yearly_cents,
    stripe_price_id_monthly = EXCLUDED.stripe_price_id_monthly, stripe_price_id_yearly = EXCLUDED.stripe_price_id_yearly
),
plan_pro AS (
  INSERT INTO managed.service_plan (service_id, slug, name, description, status, highlighted, requires_payment, values_override, entitlements, price_monthly_cents, price_yearly_cents, stripe_price_id_monthly, stripe_price_id_yearly)
  SELECT id, 'cnpg-pro', 'PostgreSQL Pro',
    'Haute disponibilité (3 replicas), sauvegarde continue PITR, support prioritaire.',
    'active', true, true,
    '{"postgres": {"replicas": 3, "size": "50Gi", "resources": {"requests": {"cpu": "250m", "memory": "512Mi"}, "limits": {"cpu": "1", "memory": "1Gi"}}}}'::jsonb,
    '[{"key": "support_level", "label": "Support", "value": "Email + astreinte"}, {"key": "gti", "label": "Temps d''intervention garanti", "value": "1h"}, {"key": "backup", "label": "Sauvegarde", "value": "Continue (PITR)"}, {"key": "storage", "label": "Stockage", "value": "50 Go"}]'::jsonb,
    2999, 32389,
    'price_1Tp1KmCMRVnojgCRCLM6JaNp', 'price_1Tp1KsCMRVnojgCReXsHLIlB'
  FROM svc
  ON CONFLICT (service_id, slug) DO UPDATE SET
    name = EXCLUDED.name, description = EXCLUDED.description, requires_payment = EXCLUDED.requires_payment,
    price_monthly_cents = EXCLUDED.price_monthly_cents, price_yearly_cents = EXCLUDED.price_yearly_cents,
    stripe_price_id_monthly = EXCLUDED.stripe_price_id_monthly, stripe_price_id_yearly = EXCLUDED.stripe_price_id_yearly
)
INSERT INTO managed.service_version (service_id, chart_version, app_version, oci_reference, connection_info_schema, configurable_values_schema, ui_schema)
SELECT id, '0.1.0', '18', 'oci://registry.gitlab.com/getbunker-france-nuage/france-nuage/charts/postgres',
  '{"secrets": [{"ref": "{release}-postgres-credentials", "keys": ["username", "password"]}], "fields": [{"key": "host", "label": "Hôte", "value": "{release}-rw.{namespace}.svc.cluster.local", "display": "text", "order": 1}, {"key": "port", "label": "Port", "value": "5432", "display": "text", "order": 2}, {"key": "database", "label": "Base de données", "value": "app", "display": "text", "order": 3}, {"key": "username", "label": "Utilisateur", "source": "{release}-postgres-credentials/username", "display": "text", "order": 4}, {"key": "password", "label": "Mot de passe", "source": "{release}-postgres-credentials/password", "display": "password", "order": 5}, {"key": "connectionString", "label": "Connection string", "template": "postgresql://{username}:{password}@{host}:{port}/{database}", "display": "copy", "order": 6}]}'::jsonb,
  '{"$schema": "http://json-schema.org/draft-07/schema#", "type": "object", "properties": {"database": {"type": "string", "default": "app", "title": "Nom de la base", "description": "Nom de la base de données à créer."}, "username": {"type": "string", "default": "app", "title": "Utilisateur", "description": "Nom d''utilisateur proprietaire de la base."}}}'::jsonb,
  '{"ui:order": ["database", "username"], "database": {"ui:placeholder": "app"}, "username": {"ui:placeholder": "app"}}'::jsonb
FROM svc
ON CONFLICT (service_id, chart_version) DO UPDATE SET
  app_version = EXCLUDED.app_version,
  oci_reference = EXCLUDED.oci_reference,
  connection_info_schema = EXCLUDED.connection_info_schema,
  configurable_values_schema = EXCLUDED.configurable_values_schema,
  ui_schema = EXCLUDED.ui_schema;

COMMIT;

-- Seed managed services catalog: GitLab Runner
BEGIN;

WITH svc AS (
  INSERT INTO managed.service (slug, name, description, category, deploy_target)
  VALUES (
    'gitlab-runner',
    'GitLab Runner',
    'Runner CI/CD GitLab sur Kubernetes. Execution de pipelines GitLab directement sur l''infrastructure France Nuage avec isolation par namespace, BuildKit rootless et support Docker-in-Docker.',
    'automation',
    '{"availability": "fr"}'::jsonb
  )
  ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    category = EXCLUDED.category,
    deploy_target = EXCLUDED.deploy_target
  RETURNING id
),
plan_standard AS (
  INSERT INTO managed.service_plan (service_id, slug, name, description, status, highlighted, requires_payment, values_override, entitlements, price_monthly_cents, price_yearly_cents, stripe_price_id_monthly, stripe_price_id_yearly)
  SELECT id, 'gitlab-runner-standard', 'GitLab Runner Standard',
    'Runner CI/CD avec 3 jobs concurrents, support Docker et Kubernetes.',
    'active', false, true,
    '{"gitlab-runner": {"runners": {"secret": "gitlab-runner-secrets", "runUntagged": true, "locked": false, "limit": 5, "executor": "kubernetes"}, "concurrent": 3, "replicas": 1, "resources": {"limits": {"cpu": "1000m", "memory": "1024Mi"}, "requests": {"cpu": "500m", "memory": "512Mi"}}}}'::jsonb,
    '[{"key": "concurrent_jobs", "label": "Jobs concurrents", "value": "3"}, {"key": "support_level", "label": "Support", "value": "Email"}, {"key": "executor", "label": "Executeur", "value": "Kubernetes"}]'::jsonb,
    999, 10789,
    'price_1TrZDKCMRVnojgCReoTRn2bk', 'price_1TrZDUCMRVnojgCRIhxX2Wfs'
  FROM svc
  ON CONFLICT (service_id, slug) DO UPDATE SET
    name = EXCLUDED.name, description = EXCLUDED.description, requires_payment = EXCLUDED.requires_payment,
    values_override = EXCLUDED.values_override, entitlements = EXCLUDED.entitlements,
    price_monthly_cents = EXCLUDED.price_monthly_cents, price_yearly_cents = EXCLUDED.price_yearly_cents,
    stripe_price_id_monthly = EXCLUDED.stripe_price_id_monthly, stripe_price_id_yearly = EXCLUDED.stripe_price_id_yearly
),
plan_pro AS (
  INSERT INTO managed.service_plan (service_id, slug, name, description, status, highlighted, requires_payment, values_override, entitlements, price_monthly_cents, price_yearly_cents, stripe_price_id_monthly, stripe_price_id_yearly)
  SELECT id, 'gitlab-runner-pro', 'GitLab Runner Pro',
    'Runner CI/CD haute performance avec 10 jobs concurrents et support prioritaire.',
    'active', true, true,
    '{"gitlab-runner": {"runners": {"secret": "gitlab-runner-secrets", "runUntagged": true, "locked": false, "limit": 15, "executor": "kubernetes"}, "concurrent": 10, "replicas": 2, "resources": {"limits": {"cpu": "2000m", "memory": "2048Mi"}, "requests": {"cpu": "1000m", "memory": "1024Mi"}}}}'::jsonb,
    '[{"key": "concurrent_jobs", "label": "Jobs concurrents", "value": "10"}, {"key": "support_level", "label": "Support", "value": "Email + astreinte"}, {"key": "executor", "label": "Executeur", "value": "Kubernetes"}]'::jsonb,
    2499, 26989,
    'price_1TrZDeCMRVnojgCREx9xuafs', 'price_1TrZDnCMRVnojgCRbrXrnwyp'
  FROM svc
  ON CONFLICT (service_id, slug) DO UPDATE SET
    name = EXCLUDED.name, description = EXCLUDED.description, requires_payment = EXCLUDED.requires_payment,
    values_override = EXCLUDED.values_override, entitlements = EXCLUDED.entitlements,
    price_monthly_cents = EXCLUDED.price_monthly_cents, price_yearly_cents = EXCLUDED.price_yearly_cents,
    stripe_price_id_monthly = EXCLUDED.stripe_price_id_monthly, stripe_price_id_yearly = EXCLUDED.stripe_price_id_yearly
)
INSERT INTO managed.service_version (service_id, chart_version, app_version, oci_reference, configurable_values_schema, ui_schema)
SELECT id, '0.2.0', '18.9.0', 'oci://registry.gitlab.com/getbunker-france-nuage/france-nuage/charts/gitlab-runner',
  '{"$schema": "http://json-schema.org/draft-07/schema#", "type": "object", "required": ["runner-token"], "properties": {"runner-token": {"type": "string", "title": "Token du runner", "description": "Token d''authentification GitLab Runner (glrt-xxxx). A recuperer depuis GitLab > Settings > CI/CD > Runners > New project runner.", "format": "password"}, "gitlab-runner": {"type": "object", "properties": {"runners": {"type": "object", "properties": {"name": {"type": "string", "default": "frn-kubernetes-runner", "title": "Nom du runner", "description": "Nom affiche dans l''interface GitLab."}, "tags": {"type": "string", "default": "kubernetes,docker", "title": "Tags", "description": "Tags du runner, separes par des virgules. Utilises dans .gitlab-ci.yml pour cibler ce runner."}}}}}}}'::jsonb,
  '{"ui:order": ["runner-token", "gitlab-runner"], "runner-token": {"ui:placeholder": "glrt-xxxxxxxxxxxxxxxxxxxx"}, "gitlab-runner": {"ui:options": {"label": false}, "runners": {"ui:options": {"label": false}, "name": {"ui:placeholder": "frn-kubernetes-runner"}, "tags": {"ui:placeholder": "kubernetes,docker"}}}}'::jsonb
FROM svc
ON CONFLICT (service_id, chart_version) DO UPDATE SET
  app_version = EXCLUDED.app_version,
  oci_reference = EXCLUDED.oci_reference,
  configurable_values_schema = EXCLUDED.configurable_values_schema,
  ui_schema = EXCLUDED.ui_schema;

COMMIT;
