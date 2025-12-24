-- Migration: Create Network Tables (VPC, VNet, IPAM, Security Groups)
-- This migration implements the new SDN-based network architecture

-- Sequence for VXLAN tag allocation
CREATE SEQUENCE IF NOT EXISTS vxlan_tag_seq START WITH 100 INCREMENT BY 1 MAXVALUE 16777215;

-- VPCs table (replaces zero_trust_networks)
CREATE TABLE "public"."vpcs" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "name" text NOT NULL,
    "slug" text NOT NULL,
    "organization_id" uuid NOT NULL,
    "region" text NOT NULL DEFAULT 'fr-paris-1',
    "sdn_zone_id" text NULL,
    "vxlan_tag" integer NOT NULL,
    "state" text NOT NULL DEFAULT 'PENDING',
    "mtu" integer NOT NULL DEFAULT 1450,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "vpcs_organization_id_fkey" FOREIGN KEY ("organization_id")
        REFERENCES "public"."organizations" ("id") ON DELETE CASCADE,
    CONSTRAINT "vpcs_slug_unique" UNIQUE ("slug"),
    CONSTRAINT "vpcs_vxlan_tag_unique" UNIQUE ("vxlan_tag"),
    CONSTRAINT "vpcs_sdn_zone_id_unique" UNIQUE ("sdn_zone_id"),
    CONSTRAINT "vpcs_vxlan_tag_range" CHECK (vxlan_tag >= 1 AND vxlan_tag <= 16777215),
    CONSTRAINT "vpcs_mtu_range" CHECK (mtu >= 1280 AND mtu <= 1500),
    CONSTRAINT "vpcs_state_valid" CHECK (state IN ('PENDING', 'CREATING', 'ACTIVE', 'ERROR', 'DELETING'))
);

CREATE INDEX "idx_vpcs_organization_id" ON "public"."vpcs" ("organization_id");
CREATE INDEX "idx_vpcs_state" ON "public"."vpcs" ("state");

-- VNets table (replaces zero_trust_network_types conceptually)
CREATE TABLE "public"."vnets" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "vpc_id" uuid NOT NULL,
    "name" text NOT NULL,
    "vnet_bridge_id" text NULL,
    "subnet" cidr NOT NULL,
    "gateway" inet NOT NULL,
    "dhcp_enabled" boolean NOT NULL DEFAULT false,
    "dns_servers" text NULL,
    "state" text NOT NULL DEFAULT 'PENDING',
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "vnets_vpc_id_fkey" FOREIGN KEY ("vpc_id")
        REFERENCES "public"."vpcs" ("id") ON DELETE CASCADE,
    CONSTRAINT "vnets_vnet_bridge_id_unique" UNIQUE ("vnet_bridge_id"),
    CONSTRAINT "vnets_state_valid" CHECK (state IN ('PENDING', 'ACTIVE', 'ERROR'))
);

CREATE INDEX "idx_vnets_vpc_id" ON "public"."vnets" ("vpc_id");

-- Security Groups table
CREATE TABLE "public"."security_groups" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "vpc_id" uuid NOT NULL,
    "name" text NOT NULL,
    "description" text NULL,
    "is_default" boolean NOT NULL DEFAULT false,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    "updated_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "security_groups_vpc_id_fkey" FOREIGN KEY ("vpc_id")
        REFERENCES "public"."vpcs" ("id") ON DELETE CASCADE,
    CONSTRAINT "security_groups_name_vpc_unique" UNIQUE ("vpc_id", "name")
);

CREATE INDEX "idx_security_groups_vpc_id" ON "public"."security_groups" ("vpc_id");

-- Security Rules table
CREATE TABLE "public"."security_rules" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "security_group_id" uuid NOT NULL,
    "direction" text NOT NULL,
    "protocol" text NOT NULL,
    "port_from" integer NULL,
    "port_to" integer NULL,
    "source_cidr" cidr NOT NULL DEFAULT '0.0.0.0/0',
    "action" text NOT NULL,
    "priority" integer NOT NULL DEFAULT 100,
    "description" text NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "security_rules_security_group_id_fkey" FOREIGN KEY ("security_group_id")
        REFERENCES "public"."security_groups" ("id") ON DELETE CASCADE,
    CONSTRAINT "security_rules_direction_valid" CHECK (direction IN ('INBOUND', 'OUTBOUND')),
    CONSTRAINT "security_rules_protocol_valid" CHECK (protocol IN ('TCP', 'UDP', 'ICMP', 'ALL')),
    CONSTRAINT "security_rules_action_valid" CHECK (action IN ('ALLOW', 'DENY')),
    CONSTRAINT "security_rules_priority_range" CHECK (priority >= 1 AND priority <= 65535),
    CONSTRAINT "security_rules_port_range" CHECK (
        (port_from IS NULL AND port_to IS NULL) OR
        (port_from >= 0 AND port_from <= 65535 AND port_to >= 0 AND port_to <= 65535)
    )
);

CREATE INDEX "idx_security_rules_security_group_id" ON "public"."security_rules" ("security_group_id");

-- Instance Interfaces table (join table for instances to vnets)
CREATE TABLE "public"."instance_interfaces" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "instance_id" uuid NOT NULL,
    "vnet_id" uuid NOT NULL,
    "ip_address_id" uuid NULL,
    "mac_address" macaddr NOT NULL,
    "device_name" text NOT NULL DEFAULT 'net0',
    "driver" text NOT NULL DEFAULT 'virtio',
    "firewall_enabled" boolean NOT NULL DEFAULT true,
    "rate_limit_mbps" integer NULL,
    "state" text NOT NULL DEFAULT 'ATTACHED',
    "created_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "instance_interfaces_instance_id_fkey" FOREIGN KEY ("instance_id")
        REFERENCES "public"."instances" ("id") ON DELETE CASCADE,
    CONSTRAINT "instance_interfaces_vnet_id_fkey" FOREIGN KEY ("vnet_id")
        REFERENCES "public"."vnets" ("id") ON DELETE CASCADE,
    CONSTRAINT "instance_interfaces_mac_address_unique" UNIQUE ("mac_address"),
    CONSTRAINT "instance_interfaces_state_valid" CHECK (state IN ('ATTACHED', 'DETACHED'))
);

CREATE INDEX "idx_instance_interfaces_instance_id" ON "public"."instance_interfaces" ("instance_id");
CREATE INDEX "idx_instance_interfaces_vnet_id" ON "public"."instance_interfaces" ("vnet_id");

-- IP Addresses table (IPAM)
CREATE TABLE "public"."ip_addresses" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "vnet_id" uuid NOT NULL,
    "address" inet NOT NULL,
    "instance_interface_id" uuid NULL,
    "mac_address" macaddr NOT NULL,
    "allocation_type" text NOT NULL DEFAULT 'STATIC',
    "hostname" text NULL,
    "allocated_at" timestamptz NULL,
    "released_at" timestamptz NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "ip_addresses_vnet_id_fkey" FOREIGN KEY ("vnet_id")
        REFERENCES "public"."vnets" ("id") ON DELETE CASCADE,
    CONSTRAINT "ip_addresses_instance_interface_id_fkey" FOREIGN KEY ("instance_interface_id")
        REFERENCES "public"."instance_interfaces" ("id") ON DELETE SET NULL,
    CONSTRAINT "ip_addresses_address_vnet_unique" UNIQUE ("vnet_id", "address"),
    CONSTRAINT "ip_addresses_mac_address_unique" UNIQUE ("mac_address"),
    CONSTRAINT "ip_addresses_allocation_type_valid"
        CHECK (allocation_type IN ('STATIC', 'DYNAMIC', 'RESERVED', 'GATEWAY'))
);

CREATE INDEX "idx_ip_addresses_vnet_id" ON "public"."ip_addresses" ("vnet_id");
CREATE INDEX "idx_ip_addresses_instance_interface_id" ON "public"."ip_addresses" ("instance_interface_id");

-- Update instance_interfaces to reference ip_addresses
ALTER TABLE "public"."instance_interfaces"
    ADD CONSTRAINT "instance_interfaces_ip_address_id_fkey"
    FOREIGN KEY ("ip_address_id") REFERENCES "public"."ip_addresses" ("id") ON DELETE SET NULL;

-- Security Group to Interface associations (many-to-many)
CREATE TABLE "public"."sg_interface_associations" (
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "security_group_id" uuid NOT NULL,
    "instance_interface_id" uuid NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id"),
    CONSTRAINT "sg_interface_associations_sg_fkey" FOREIGN KEY ("security_group_id")
        REFERENCES "public"."security_groups" ("id") ON DELETE CASCADE,
    CONSTRAINT "sg_interface_associations_interface_fkey" FOREIGN KEY ("instance_interface_id")
        REFERENCES "public"."instance_interfaces" ("id") ON DELETE CASCADE,
    CONSTRAINT "sg_interface_associations_unique" UNIQUE ("security_group_id", "instance_interface_id")
);

CREATE INDEX "idx_sg_interface_associations_sg" ON "public"."sg_interface_associations" ("security_group_id");
CREATE INDEX "idx_sg_interface_associations_interface" ON "public"."sg_interface_associations" ("instance_interface_id");
