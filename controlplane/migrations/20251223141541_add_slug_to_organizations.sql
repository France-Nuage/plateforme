-- atlas:nolint BC102
-- Add slug column to organizations table for DNS-compatible identifiers

-- Step 1: Add slug column as nullable first
ALTER TABLE "public"."organizations" ADD COLUMN "slug" text NULL;

-- Step 2: Generate slugs for existing organizations
-- Convert to lowercase, replace spaces with hyphens, remove special characters
UPDATE "public"."organizations"
SET slug = LOWER(
    REGEXP_REPLACE(
        REGEXP_REPLACE(
            REGEXP_REPLACE(name, '[^a-zA-Z0-9\s-]', '', 'g'),
            '\s+', '-', 'g'
        ),
        '-+', '-', 'g'
    )
);

-- Step 3: Trim leading/trailing hyphens
UPDATE "public"."organizations"
SET slug = TRIM(BOTH '-' FROM slug);

-- Step 4: Handle potential duplicates by appending row number
WITH duplicates AS (
    SELECT id, slug, ROW_NUMBER() OVER (PARTITION BY slug ORDER BY created_at) as rn
    FROM "public"."organizations"
)
UPDATE "public"."organizations" o
SET slug = d.slug || '-' || d.rn
FROM duplicates d
WHERE o.id = d.id AND d.rn > 1;

-- Step 5: Handle empty slugs by using a UUID-based fallback
UPDATE "public"."organizations"
SET slug = 'org-' || SUBSTRING(id::text, 1, 8)
WHERE slug IS NULL OR slug = '';

-- Step 6: Truncate slugs to max 63 characters (DNS label limit)
UPDATE "public"."organizations"
SET slug = SUBSTRING(slug, 1, 63)
WHERE LENGTH(slug) > 63;

-- Step 7: Make column NOT NULL
ALTER TABLE "public"."organizations" ALTER COLUMN "slug" SET NOT NULL;

-- Step 8: Add unique constraint
CREATE UNIQUE INDEX "organizations_slug_idx" ON "public"."organizations" ("slug");
