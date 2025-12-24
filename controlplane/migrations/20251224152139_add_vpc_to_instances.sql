-- Migration: Add VPC and VNet columns to instances table
-- These columns link instances to the new network architecture

-- Add vpc_id column to instances (nullable during migration)
ALTER TABLE "public"."instances" ADD COLUMN "vpc_id" uuid NULL;
ALTER TABLE "public"."instances" ADD COLUMN "vnet_id" uuid NULL;
ALTER TABLE "public"."instances" ADD COLUMN "mac_address" macaddr NULL;

-- Add foreign key constraints
ALTER TABLE "public"."instances"
    ADD CONSTRAINT "instances_vpc_id_fkey"
    FOREIGN KEY ("vpc_id") REFERENCES "public"."vpcs" ("id") ON DELETE SET NULL;

ALTER TABLE "public"."instances"
    ADD CONSTRAINT "instances_vnet_id_fkey"
    FOREIGN KEY ("vnet_id") REFERENCES "public"."vnets" ("id") ON DELETE SET NULL;

-- Create indexes for performance
CREATE INDEX "idx_instances_vpc_id" ON "public"."instances" ("vpc_id");
CREATE INDEX "idx_instances_vnet_id" ON "public"."instances" ("vnet_id");
