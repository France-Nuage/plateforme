-- Migration: Fix mac_address column type
-- Change from macaddr to text for compatibility with sqlx String binding

ALTER TABLE "public"."instances"
    ALTER COLUMN "mac_address" TYPE text USING mac_address::text;
