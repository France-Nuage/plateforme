-- Migration: Fix ip_addresses address column type
-- Change from inet to text for compatibility with sqlx String binding
-- Validation of inet format is handled in application code via cast on insert

-- First drop the unique constraint that includes the address column
ALTER TABLE "public"."ip_addresses"
    DROP CONSTRAINT IF EXISTS "ip_addresses_address_vnet_unique";

-- Change address from inet to text (preserves the data as string representation)
ALTER TABLE "public"."ip_addresses"
    ALTER COLUMN "address" TYPE text USING address::text;

-- Recreate the unique constraint with text column
ALTER TABLE "public"."ip_addresses"
    ADD CONSTRAINT "ip_addresses_address_vnet_unique" UNIQUE ("vnet_id", "address");
