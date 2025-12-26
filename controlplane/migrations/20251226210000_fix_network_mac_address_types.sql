-- Migration: Fix mac_address column types in network tables
-- Change from macaddr to text for compatibility with sqlx String binding

-- Change ip_addresses.mac_address from macaddr to text
-- First drop the unique constraint, change the type, then recreate the constraint
ALTER TABLE "public"."ip_addresses"
    DROP CONSTRAINT IF EXISTS "ip_addresses_mac_address_unique";

ALTER TABLE "public"."ip_addresses"
    ALTER COLUMN "mac_address" TYPE text USING mac_address::text;

-- Make mac_address nullable since gateways don't have MAC addresses
ALTER TABLE "public"."ip_addresses"
    ALTER COLUMN "mac_address" DROP NOT NULL;

-- Create a partial unique index (only on non-null values)
CREATE UNIQUE INDEX "ip_addresses_mac_address_unique"
    ON "public"."ip_addresses" ("mac_address")
    WHERE mac_address IS NOT NULL;

-- Change instance_interfaces.mac_address from macaddr to text
ALTER TABLE "public"."instance_interfaces"
    DROP CONSTRAINT IF EXISTS "instance_interfaces_mac_address_unique";

ALTER TABLE "public"."instance_interfaces"
    ALTER COLUMN "mac_address" TYPE text USING mac_address::text;

CREATE UNIQUE INDEX "instance_interfaces_mac_address_unique"
    ON "public"."instance_interfaces" ("mac_address");
