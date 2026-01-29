-- Migration: Create networks and addresses tables for SDN management
-- This migration replaces the zero_trust_network system with a proper VPC network model

-- Create "networks" table
-- Represents a VPC Network resource that connects instances and enables client isolation
CREATE TABLE "public"."networks" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "name" character varying(63) NOT NULL,
  "description" text NULL,
  "ipv4_range" character varying(18) NOT NULL, -- CIDR format, e.g., "192.168.0.0/16"
  "gateway_ipv4" character varying(15) NULL, -- Gateway IP, e.g., "192.168.0.1"
  "mtu" integer NOT NULL DEFAULT 1450, -- Default optimized for VXLAN
  "project_id" uuid NOT NULL,
  "proxmox_zone_id" character varying(64) NULL, -- Proxmox SDN Zone ID
  "proxmox_vnet_id" character varying(64) NULL, -- Proxmox SDN VNet ID
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "networks_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "networks_name_project_unique" UNIQUE ("name", "project_id"),
  CONSTRAINT "networks_mtu_check" CHECK (mtu >= 1300 AND mtu <= 8896)
);

-- Create index for project lookups
CREATE INDEX "idx_networks_project_id" ON "public"."networks" ("project_id");

-- Create "addresses" table
-- Represents IP address reservations within a network (IPAM)
CREATE TABLE "public"."addresses" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "address" character varying(15) NOT NULL, -- IPv4 address
  "name" character varying(63) NULL,
  "description" text NULL,
  "status" character varying(20) NOT NULL DEFAULT 'RESERVED', -- RESERVING, RESERVED, IN_USE
  "network_id" uuid NOT NULL,
  "instance_id" uuid NULL, -- The instance using this address (if IN_USE)
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "addresses_network_id_fkey" FOREIGN KEY ("network_id") REFERENCES "public"."networks" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "addresses_instance_id_fkey" FOREIGN KEY ("instance_id") REFERENCES "public"."instances" ("id") ON UPDATE NO ACTION ON DELETE SET NULL,
  CONSTRAINT "addresses_address_network_unique" UNIQUE ("address", "network_id")
);

-- Create index for network lookups
CREATE INDEX "idx_addresses_network_id" ON "public"."addresses" ("network_id");
-- Create index for instance lookups
CREATE INDEX "idx_addresses_instance_id" ON "public"."addresses" ("instance_id");
-- Create index for finding available addresses
CREATE INDEX "idx_addresses_status" ON "public"."addresses" ("status");

-- Create "instance_network_interfaces" table
-- Junction table linking instances to networks with their assigned IP
CREATE TABLE "public"."instance_network_interfaces" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "instance_id" uuid NOT NULL,
  "network_id" uuid NOT NULL,
  "address_id" uuid NULL, -- The assigned address (may be NULL during allocation)
  "name" character varying(63) NULL, -- Interface name, e.g., "eth0"
  "created_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "instance_network_interfaces_instance_id_fkey" FOREIGN KEY ("instance_id") REFERENCES "public"."instances" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "instance_network_interfaces_network_id_fkey" FOREIGN KEY ("network_id") REFERENCES "public"."networks" ("id") ON UPDATE NO ACTION ON DELETE RESTRICT,
  CONSTRAINT "instance_network_interfaces_address_id_fkey" FOREIGN KEY ("address_id") REFERENCES "public"."addresses" ("id") ON UPDATE NO ACTION ON DELETE SET NULL,
  CONSTRAINT "instance_network_interfaces_instance_network_unique" UNIQUE ("instance_id", "network_id")
);

-- Create index for instance lookups
CREATE INDEX "idx_instance_network_interfaces_instance_id" ON "public"."instance_network_interfaces" ("instance_id");
-- Create index for network lookups (to check if network has active instances)
CREATE INDEX "idx_instance_network_interfaces_network_id" ON "public"."instance_network_interfaces" ("network_id");

-- Remove zero_trust_network_id column from instances
ALTER TABLE "public"."instances" DROP CONSTRAINT IF EXISTS "instances_zero_trust_network_id_fkey";
ALTER TABLE "public"."instances" DROP COLUMN IF EXISTS "zero_trust_network_id";

-- Drop zero_trust tables
DROP TABLE IF EXISTS "public"."zero_trust_networks";
DROP TABLE IF EXISTS "public"."zero_trust_network_types";
