ALTER TABLE hypervisors ADD COLUMN organization_id UUID NOT NULL REFERENCES organizations (id) ON DELETE CASCADE;

