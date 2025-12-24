-- Migration: Migrate ZeroTrustNetwork data to VPC architecture
-- This migration converts existing ZeroTrustNetwork records to VPCs and VNets

-- Function to generate a valid slug from a name
CREATE OR REPLACE FUNCTION generate_slug(input_name text)
RETURNS text AS $$
BEGIN
    RETURN LOWER(
        REGEXP_REPLACE(
            REGEXP_REPLACE(
                REGEXP_REPLACE(input_name, '[^a-zA-Z0-9]+', '-', 'g'),
                '^-+|-+$', '', 'g'
            ),
            '-+', '-', 'g'
        )
    );
END;
$$ LANGUAGE plpgsql;

-- Migrate ZeroTrustNetworks to VPCs
INSERT INTO vpcs (id, name, slug, organization_id, vxlan_tag, state, created_at, updated_at)
SELECT
    ztn.id,
    ztn.name,
    generate_slug(ztn.name) || '-' || SUBSTRING(ztn.id::text FROM 1 FOR 8),
    ztn.organization_id,
    nextval('vxlan_tag_seq'),
    'ACTIVE',
    ztn.created_at,
    ztn.updated_at
FROM zero_trust_networks ztn
ON CONFLICT (id) DO NOTHING;

-- Migrate ZeroTrustNetworkTypes to VNets (linked to first VPC of the same organization via ZTN)
-- Each ZTN type becomes a VNet, linked to VPCs that referenced it
INSERT INTO vnets (id, vpc_id, name, subnet, gateway, state, vnet_bridge_id, created_at, updated_at)
SELECT DISTINCT ON (ztnt.id, ztn.id)
    gen_random_uuid(),
    ztn.id,  -- vpc_id is the migrated ZTN id
    ztnt.name,
    '10.0.0.0/24'::cidr,  -- Default subnet, should be adjusted per deployment
    '10.0.0.1'::inet,     -- Default gateway
    'ACTIVE',
    'vnet-' || generate_slug(ztn.name) || '-' || SUBSTRING(ztn.id::text FROM 1 FOR 4),
    ztnt.created_at,
    ztnt.updated_at
FROM zero_trust_network_types ztnt
JOIN zero_trust_networks ztn ON ztn.zero_trust_network_type_id = ztnt.id
ON CONFLICT DO NOTHING;

-- Update instances to reference VPCs (vpc_id = old zero_trust_network_id which is now a VPC id)
UPDATE instances i
SET vpc_id = i.zero_trust_network_id
WHERE i.zero_trust_network_id IS NOT NULL;

-- Set vnet_id to the first VNet of the VPC for existing instances
UPDATE instances i
SET vnet_id = (
    SELECT v.id FROM vnets v
    WHERE v.vpc_id = i.vpc_id
    LIMIT 1
)
WHERE i.vpc_id IS NOT NULL AND i.vnet_id IS NULL;

-- Create default security groups for each VPC
INSERT INTO security_groups (vpc_id, name, description, is_default)
SELECT id, 'default', 'Default security group - DENY ALL', true
FROM vpcs
ON CONFLICT (vpc_id, name) DO NOTHING;

-- Create default DENY ALL INBOUND rule for default security groups
INSERT INTO security_rules (security_group_id, direction, protocol, source_cidr, action, priority, description)
SELECT sg.id, 'INBOUND', 'ALL', '0.0.0.0/0', 'DENY', 65535, 'Default deny all inbound'
FROM security_groups sg
WHERE sg.is_default = true
  AND NOT EXISTS (
    SELECT 1 FROM security_rules sr
    WHERE sr.security_group_id = sg.id
      AND sr.direction = 'INBOUND'
      AND sr.action = 'DENY'
      AND sr.priority = 65535
  );

-- Create default DENY ALL OUTBOUND rule for default security groups
INSERT INTO security_rules (security_group_id, direction, protocol, source_cidr, action, priority, description)
SELECT sg.id, 'OUTBOUND', 'ALL', '0.0.0.0/0', 'DENY', 65535, 'Default deny all outbound'
FROM security_groups sg
WHERE sg.is_default = true
  AND NOT EXISTS (
    SELECT 1 FROM security_rules sr
    WHERE sr.security_group_id = sg.id
      AND sr.direction = 'OUTBOUND'
      AND sr.action = 'DENY'
      AND sr.priority = 65535
  );

-- Cleanup: drop the helper function
DROP FUNCTION IF EXISTS generate_slug(text);

-- Note: We do NOT drop the old zero_trust_networks tables yet to allow rollback
-- They will be dropped in a future migration after confirming the migration is successful
