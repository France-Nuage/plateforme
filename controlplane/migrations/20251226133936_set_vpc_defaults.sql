-- Migration: Set default values for VPC mtu and vxlan_tag
-- This ensures VPC records created without explicit values are valid

-- Set default for vxlan_tag to use the sequence
ALTER TABLE "public"."vpcs"
    ALTER COLUMN "vxlan_tag" SET DEFAULT nextval('vxlan_tag_seq');
