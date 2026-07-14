-- atlas:nolint
-- Migrate organizations and projects from UUID primary keys to slug (CITEXT)
-- primary keys. All foreign key references are updated accordingly.
--
-- Phase 0: setup
-- Phase 1: prepare slug columns on all tables (while UUIDs still exist)
-- Phase 2: populate new columns via JOINs on old UUIDs
-- Phase 3: drop old FK constraints, columns, PKs
-- Phase 4: set new PKs, CHECK constraints, FK constraints, indexes
-- Phase 5: recreate views

-- ============================================================
-- PHASE 0: Setup
-- ============================================================

CREATE EXTENSION IF NOT EXISTS citext;

DROP VIEW IF EXISTS managed.service_instance_view;

-- ============================================================
-- PHASE 1: Prepare slug columns on all tables
-- ============================================================

-- 1a. Convert organizations.slug from TEXT to CITEXT
ALTER TABLE organizations ALTER COLUMN slug TYPE citext;

-- 1b. Sanitize existing organization slugs: strip digits, collapse hyphens, trim
UPDATE organizations SET slug = LOWER(TRIM(BOTH '-' FROM
    REGEXP_REPLACE(
        REGEXP_REPLACE(slug, '[^a-zA-Z-]', '', 'g'),
        '-+', '-', 'g'
    )
));
UPDATE organizations SET slug = 'unnamed' WHERE slug IS NULL OR slug = '';

-- 1c. Deduplicate organization slugs (letter suffix: -a, -b, ..., -z, -aa, -ab, ...)
WITH duplicates AS (
    SELECT ctid, slug,
           ROW_NUMBER() OVER (PARTITION BY slug ORDER BY created_at) AS rn
    FROM organizations
)
UPDATE organizations o
SET slug = SUBSTRING(d.slug, 1, 46) || '-'
    || CASE WHEN d.rn <= 27 THEN chr((95 + d.rn)::int)
            ELSE chr(((96 + (d.rn - 2) / 26))::int) || chr(((97 + (d.rn - 2) % 26))::int)
       END
FROM duplicates d
WHERE o.ctid = d.ctid AND d.rn > 1;

-- 1d. Truncate to 49 chars and clean trailing hyphens
UPDATE organizations SET slug = RTRIM(SUBSTRING(slug, 1, 49), '-')
WHERE length(slug) > 49 OR slug LIKE '%-';

-- 1e. Add organization_slug columns to dependent tables
ALTER TABLE projects ADD COLUMN organization_slug citext;
ALTER TABLE hypervisors ADD COLUMN organization_slug citext;
ALTER TABLE zero_trust_networks ADD COLUMN organization_slug citext;
ALTER TABLE organization_user ADD COLUMN organization_slug citext;
ALTER TABLE organization_service_account ADD COLUMN organization_slug citext;
ALTER TABLE invitations ADD COLUMN organization_slug citext;
ALTER TABLE organizations ADD COLUMN parent_slug citext;
ALTER TABLE managed.service_instance ADD COLUMN organization_slug citext;

-- 1f. Add slug column to projects
ALTER TABLE projects ADD COLUMN slug citext;

-- 1g. Add project_slug columns to dependent tables
ALTER TABLE instances ADD COLUMN project_slug citext;
ALTER TABLE managed.service_instance ADD COLUMN project_slug citext;

-- ============================================================
-- PHASE 2: Populate new columns via JOINs on old UUIDs
-- ============================================================

-- 2a. Organization slug columns on dependents
UPDATE projects p SET organization_slug = o.slug
FROM organizations o WHERE p.organization_id = o.id;

UPDATE hypervisors h SET organization_slug = o.slug
FROM organizations o WHERE h.organization_id = o.id;

UPDATE zero_trust_networks z SET organization_slug = o.slug
FROM organizations o WHERE z.organization_id = o.id;

UPDATE organization_user ou SET organization_slug = o.slug
FROM organizations o WHERE ou.organization_id = o.id;

UPDATE organization_service_account osa SET organization_slug = o.slug
FROM organizations o WHERE osa.organization_id = o.id;

UPDATE invitations i SET organization_slug = o.slug
FROM organizations o WHERE i.organization_id = o.id;

UPDATE organizations child SET parent_slug = parent.slug
FROM organizations parent WHERE child.parent_id = parent.id;

UPDATE managed.service_instance si SET organization_slug = o.slug
FROM organizations o WHERE si.organization_id = o.id;

-- 2b. Generate project slugs: {org_slug}-{name_slug}
UPDATE projects p SET slug = LOWER(RTRIM(SUBSTRING(
    TRIM(BOTH '-' FROM
        REGEXP_REPLACE(
            REGEXP_REPLACE(
                o.slug || '-' || REGEXP_REPLACE(p.name, '[^a-zA-Z\s-]', '', 'g'),
                '\s+', '-', 'g'
            ),
            '-+', '-', 'g'
        )
    ), 1, 49), '-'))
FROM organizations o WHERE p.organization_id = o.id;

UPDATE projects SET slug = 'unnamed-project' WHERE slug IS NULL OR slug = '';

-- Deduplicate project slugs (same suffix strategy as organizations)
WITH duplicates AS (
    SELECT ctid, slug,
           ROW_NUMBER() OVER (PARTITION BY slug ORDER BY created_at) AS rn
    FROM projects
)
UPDATE projects p
SET slug = SUBSTRING(d.slug, 1, 46) || '-'
    || CASE WHEN d.rn <= 27 THEN chr((95 + d.rn)::int)
            ELSE chr(((96 + (d.rn - 2) / 26))::int) || chr(((97 + (d.rn - 2) % 26))::int)
       END
FROM duplicates d
WHERE p.ctid = d.ctid AND d.rn > 1;

-- 2c. Project slug columns on dependents
UPDATE instances i SET project_slug = p.slug
FROM projects p WHERE i.project_id = p.id;

UPDATE managed.service_instance si SET project_slug = p.slug
FROM projects p WHERE si.project_id = p.id;

-- 2d. Set NOT NULL on all new columns
ALTER TABLE projects ALTER COLUMN organization_slug SET NOT NULL;
ALTER TABLE projects ALTER COLUMN slug SET NOT NULL;
ALTER TABLE hypervisors ALTER COLUMN organization_slug SET NOT NULL;
ALTER TABLE zero_trust_networks ALTER COLUMN organization_slug SET NOT NULL;
ALTER TABLE organization_user ALTER COLUMN organization_slug SET NOT NULL;
ALTER TABLE organization_service_account ALTER COLUMN organization_slug SET NOT NULL;
ALTER TABLE invitations ALTER COLUMN organization_slug SET NOT NULL;
ALTER TABLE instances ALTER COLUMN project_slug SET NOT NULL;
ALTER TABLE managed.service_instance ALTER COLUMN organization_slug SET NOT NULL;
ALTER TABLE managed.service_instance ALTER COLUMN project_slug SET NOT NULL;

-- ============================================================
-- PHASE 3: Drop old FK constraints, columns, PKs
-- ============================================================

-- 3a. Drop FK constraints on organization_id
ALTER TABLE projects DROP CONSTRAINT projects_organization_id_fkey;
ALTER TABLE hypervisors DROP CONSTRAINT hypervisors_organization_id_fkey;
ALTER TABLE zero_trust_networks DROP CONSTRAINT zero_trust_networks_organization_id_fkey;
ALTER TABLE organization_user DROP CONSTRAINT user_organizations_organization_id_fkey;
ALTER TABLE organization_service_account DROP CONSTRAINT organization_service_account_organization_id_fkey;
ALTER TABLE invitations DROP CONSTRAINT invitations_organization_id_fkey;
ALTER TABLE organizations DROP CONSTRAINT organizations_parent_id_fkey;

-- 3b. Drop FK constraint on project_id
ALTER TABLE instances DROP CONSTRAINT instances_project_id_fkey;

-- 3c. Drop old unique indexes on junction tables
DROP INDEX IF EXISTS user_organizations_user_id_organization_id_idx;
DROP INDEX IF EXISTS organization_service_account_service_account_id_organization_id;
DROP INDEX IF EXISTS organizations_slug_idx;
DROP INDEX IF EXISTS idx_managed_service_instance_organization;
DROP INDEX IF EXISTS idx_managed_service_instance_project;

-- 3d. Drop old UUID columns from dependent tables
ALTER TABLE projects DROP COLUMN organization_id;
ALTER TABLE hypervisors DROP COLUMN organization_id;
ALTER TABLE zero_trust_networks DROP COLUMN organization_id;
ALTER TABLE organization_user DROP COLUMN organization_id;
ALTER TABLE organization_service_account DROP COLUMN organization_id;
ALTER TABLE invitations DROP COLUMN organization_id;
ALTER TABLE organizations DROP COLUMN parent_id;
ALTER TABLE instances DROP COLUMN project_id;
ALTER TABLE managed.service_instance DROP COLUMN organization_id;
ALTER TABLE managed.service_instance DROP COLUMN project_id;

-- 3e. Drop old PKs and UUID columns
ALTER TABLE organizations DROP CONSTRAINT organizations_pkey;
ALTER TABLE organizations DROP COLUMN id;

ALTER TABLE projects DROP CONSTRAINT projects_pkey;
ALTER TABLE projects DROP COLUMN id;

-- ============================================================
-- PHASE 4: New PKs, constraints, FKs, indexes
-- ============================================================

-- 4a. Organization PK and constraints
ALTER TABLE organizations ADD PRIMARY KEY (slug);
ALTER TABLE organizations ADD CONSTRAINT check_organization_slug_length
    CHECK (length(slug) <= 49);
ALTER TABLE organizations ADD CONSTRAINT check_organization_slug_pattern
    CHECK (slug ~ '^[a-zA-Z]([a-zA-Z-]*[a-zA-Z])?$');

-- 4b. Project PK and constraints
ALTER TABLE projects ADD PRIMARY KEY (slug);
ALTER TABLE projects ADD CONSTRAINT check_project_slug_length
    CHECK (length(slug) <= 49);
ALTER TABLE projects ADD CONSTRAINT check_project_slug_pattern
    CHECK (slug ~ '^[a-zA-Z]([a-zA-Z-]*[a-zA-Z])?$');

-- 4c. Organization FK constraints
ALTER TABLE projects ADD CONSTRAINT projects_organization_slug_fkey
    FOREIGN KEY (organization_slug) REFERENCES organizations(slug)
    ON UPDATE NO ACTION ON DELETE CASCADE;

ALTER TABLE hypervisors ADD CONSTRAINT hypervisors_organization_slug_fkey
    FOREIGN KEY (organization_slug) REFERENCES organizations(slug)
    ON UPDATE NO ACTION ON DELETE CASCADE;

ALTER TABLE zero_trust_networks ADD CONSTRAINT zero_trust_networks_organization_slug_fkey
    FOREIGN KEY (organization_slug) REFERENCES organizations(slug)
    ON UPDATE NO ACTION ON DELETE CASCADE;

ALTER TABLE organization_user ADD CONSTRAINT organization_user_organization_slug_fkey
    FOREIGN KEY (organization_slug) REFERENCES organizations(slug)
    ON UPDATE NO ACTION ON DELETE CASCADE;

ALTER TABLE organization_service_account ADD CONSTRAINT organization_service_account_organization_slug_fkey
    FOREIGN KEY (organization_slug) REFERENCES organizations(slug)
    ON UPDATE NO ACTION ON DELETE CASCADE;

ALTER TABLE invitations ADD CONSTRAINT invitations_organization_slug_fkey
    FOREIGN KEY (organization_slug) REFERENCES organizations(slug)
    ON UPDATE NO ACTION ON DELETE CASCADE;

ALTER TABLE organizations ADD CONSTRAINT organizations_parent_slug_fkey
    FOREIGN KEY (parent_slug) REFERENCES organizations(slug)
    ON UPDATE NO ACTION ON DELETE SET NULL;

-- 4d. Project FK constraints
ALTER TABLE instances ADD CONSTRAINT instances_project_slug_fkey
    FOREIGN KEY (project_slug) REFERENCES projects(slug)
    ON UPDATE NO ACTION ON DELETE CASCADE;

-- 4e. Recreate indexes
CREATE UNIQUE INDEX organization_user_user_id_organization_slug_idx
    ON organization_user(user_id, organization_slug);

CREATE UNIQUE INDEX organization_service_account_sa_id_organization_slug_idx
    ON organization_service_account(service_account_id, organization_slug);

CREATE INDEX idx_managed_service_instance_organization
    ON managed.service_instance(organization_slug);

CREATE INDEX idx_managed_service_instance_project
    ON managed.service_instance(project_slug);

-- ============================================================
-- PHASE 5: Recreate managed view
-- ============================================================

CREATE VIEW managed.service_instance_view AS
SELECT si.id,
       si.service_id,
       si.version_id,
       si.plan_id,
       si.project_slug,
       si.organization_slug,
       si.cluster_id,
       si.namespace,
       si.release_name,
       si.user_values,
       abs.name AS status,
       si.created_at
FROM managed.service_instance si
JOIN lib_fsm.state_machine sm ON sm.state_machine__id = si.status
JOIN lib_fsm.abstract_state abs ON abs.abstract_state__id = sm.abstract_state__id;
