-- Migration: Change VNet subnet/gateway columns from cidr/inet to text
-- This allows sqlx to properly decode these values to Rust Strings
-- Validation of CIDR/inet format is handled in application code

-- Change subnet from cidr to text (preserves the data as string representation)
ALTER TABLE "public"."vnets"
    ALTER COLUMN "subnet" TYPE text USING subnet::text;

-- Change gateway from inet to text (preserves the data as string representation)
ALTER TABLE "public"."vnets"
    ALTER COLUMN "gateway" TYPE text USING gateway::text;
