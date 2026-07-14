-- Seed the managed services catalog (postgres, gitlab-runner) with plans
-- matching the pricing published on france-nuage.fr.
--
-- Versions are NOT seeded here: production registers them through the
-- RegisterVersion gRPC call from the charts CI pipeline.

-- ============================================================
-- PostgreSQL (CNPG)
-- ============================================================

INSERT INTO managed.service (id, slug, name, description, category, database_engine, deploy_target)
VALUES (
    gen_random_uuid(),
    'postgres',
    'PostgreSQL',
    'PostgreSQL hébergé en France. On s''occupe des backups quotidiens, des mises à jour et du monitoring. Vous récupérez une chaîne de connexion et vous codez.',
    'database',
    'cnpg',
    '{"availability": "fr"}'::jsonb
)
ON CONFLICT (slug) DO UPDATE SET
    name        = EXCLUDED.name,
    description = EXCLUDED.description,
    category    = EXCLUDED.category,
    database_engine = EXCLUDED.database_engine,
    deploy_target   = EXCLUDED.deploy_target,
    deactivated_at  = NULL;

-- Pico plan (25 EUR HT/mois)
INSERT INTO managed.service_plan (
    id, service_id, slug, name, description, status, highlighted,
    values_override, entitlements,
    price_monthly_cents, price_yearly_cents,
    requires_payment, stripe_price_id_monthly, stripe_price_id_yearly
)
SELECT
    gen_random_uuid(),
    s.id,
    'postgres-pico',
    'Pico',
    'Instance PostgreSQL single-replica. Backups quotidiens, monitoring 24/7.',
    'active',
    false,
    '{"postgres": {"replicas": 1, "size": "10Gi", "resources": {"requests": {"cpu": "100m", "memory": "256Mi"}, "limits": {"cpu": "500m", "memory": "512Mi"}}}}'::jsonb,
    '[{"key": "hosting", "label": "Hébergement", "value": "France, conforme RGPD"}, {"key": "backup", "label": "Sauvegarde", "value": "Quotidienne"}, {"key": "monitoring", "label": "Monitoring", "value": "24/7"}, {"key": "migration", "label": "Migration", "value": "Sans lock-in via pg_dump"}]'::jsonb,
    2500,
    NULL,
    true,
    NULL,
    NULL
FROM managed.service s WHERE s.slug = 'postgres'
ON CONFLICT (service_id, slug) DO UPDATE SET
    name                   = EXCLUDED.name,
    description            = EXCLUDED.description,
    status                 = EXCLUDED.status,
    highlighted            = EXCLUDED.highlighted,
    values_override        = EXCLUDED.values_override,
    entitlements           = EXCLUDED.entitlements,
    price_monthly_cents    = EXCLUDED.price_monthly_cents,
    price_yearly_cents     = EXCLUDED.price_yearly_cents,
    requires_payment       = EXCLUDED.requires_payment,
    stripe_price_id_monthly = EXCLUDED.stripe_price_id_monthly,
    stripe_price_id_yearly  = EXCLUDED.stripe_price_id_yearly;

-- Nano plan (50 EUR HT/mois)
INSERT INTO managed.service_plan (
    id, service_id, slug, name, description, status, highlighted,
    values_override, entitlements,
    price_monthly_cents, price_yearly_cents,
    requires_payment, stripe_price_id_monthly, stripe_price_id_yearly
)
SELECT
    gen_random_uuid(),
    s.id,
    'postgres-nano',
    'Nano',
    'Instance PostgreSQL haute disponibilité (3 replicas). Sauvegarde continue PITR, monitoring 24/7.',
    'active',
    true,
    '{"postgres": {"replicas": 3, "size": "50Gi", "resources": {"requests": {"cpu": "250m", "memory": "512Mi"}, "limits": {"cpu": "1", "memory": "1Gi"}}}}'::jsonb,
    '[{"key": "hosting", "label": "Hébergement", "value": "France, conforme RGPD"}, {"key": "backup", "label": "Sauvegarde", "value": "Continue (PITR)"}, {"key": "monitoring", "label": "Monitoring", "value": "24/7"}, {"key": "migration", "label": "Migration", "value": "Sans lock-in via pg_dump"}]'::jsonb,
    5000,
    NULL,
    true,
    NULL,
    NULL
FROM managed.service s WHERE s.slug = 'postgres'
ON CONFLICT (service_id, slug) DO UPDATE SET
    name                   = EXCLUDED.name,
    description            = EXCLUDED.description,
    status                 = EXCLUDED.status,
    highlighted            = EXCLUDED.highlighted,
    values_override        = EXCLUDED.values_override,
    entitlements           = EXCLUDED.entitlements,
    price_monthly_cents    = EXCLUDED.price_monthly_cents,
    price_yearly_cents     = EXCLUDED.price_yearly_cents,
    requires_payment       = EXCLUDED.requires_payment,
    stripe_price_id_monthly = EXCLUDED.stripe_price_id_monthly,
    stripe_price_id_yearly  = EXCLUDED.stripe_price_id_yearly;

-- Archive old plans that no longer exist on the site
UPDATE managed.service_plan SET status = 'archived'
WHERE slug IN ('cnpg-standard', 'cnpg-pro')
  AND service_id = (SELECT id FROM managed.service WHERE slug = 'postgres');

-- ============================================================
-- GitLab Runner
-- ============================================================

INSERT INTO managed.service (id, slug, name, description, category, deploy_target)
VALUES (
    gen_random_uuid(),
    'gitlab-runner',
    'GitLab Runner',
    'Runners Kubernetes managés pour vos pipelines GitLab CI. BuildKit rootless, pas de Docker-in-Docker, isolation multi-tenant.',
    'automation',
    '{"availability": "fr"}'::jsonb
)
ON CONFLICT (slug) DO UPDATE SET
    name        = EXCLUDED.name,
    description = EXCLUDED.description,
    category    = EXCLUDED.category,
    deploy_target = EXCLUDED.deploy_target,
    deactivated_at = NULL;

-- Single plan (50 EUR HT/mois)
INSERT INTO managed.service_plan (
    id, service_id, slug, name, description, status, highlighted,
    values_override, entitlements,
    price_monthly_cents, price_yearly_cents,
    requires_payment, stripe_price_id_monthly, stripe_price_id_yearly
)
SELECT
    gen_random_uuid(),
    s.id,
    'gitlab-runner-standard',
    'GitLab Runner',
    'Runner CI/CD Kubernetes managé. BuildKit rootless, cache registry, isolation multi-tenant.',
    'active',
    false,
    '{"gitlab-runner": {"runners": {"runUntagged": true, "locked": false, "limit": 5, "executor": "kubernetes"}, "concurrent": 3, "replicas": 1, "resources": {"limits": {"cpu": "1000m", "memory": "1024Mi"}, "requests": {"cpu": "500m", "memory": "512Mi"}}}}'::jsonb,
    '[{"key": "buildkit", "label": "Build", "value": "BuildKit rootless sécurisé"}, {"key": "multiplatform", "label": "Plateformes", "value": "Builds multi-plateformes"}, {"key": "cache", "label": "Cache", "value": "Cache registry intégré"}, {"key": "isolation", "label": "Isolation", "value": "Multi-tenant"}]'::jsonb,
    5000,
    NULL,
    true,
    NULL,
    NULL
FROM managed.service s WHERE s.slug = 'gitlab-runner'
ON CONFLICT (service_id, slug) DO UPDATE SET
    name                   = EXCLUDED.name,
    description            = EXCLUDED.description,
    status                 = EXCLUDED.status,
    highlighted            = EXCLUDED.highlighted,
    values_override        = EXCLUDED.values_override,
    entitlements           = EXCLUDED.entitlements,
    price_monthly_cents    = EXCLUDED.price_monthly_cents,
    price_yearly_cents     = EXCLUDED.price_yearly_cents,
    requires_payment       = EXCLUDED.requires_payment,
    stripe_price_id_monthly = EXCLUDED.stripe_price_id_monthly,
    stripe_price_id_yearly  = EXCLUDED.stripe_price_id_yearly;

-- Archive old plans that no longer exist on the site
UPDATE managed.service_plan SET status = 'archived'
WHERE slug IN ('gitlab-runner-pro')
  AND service_id = (SELECT id FROM managed.service WHERE slug = 'gitlab-runner');
