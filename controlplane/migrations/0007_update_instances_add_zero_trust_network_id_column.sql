ALTER TABLE instances ADD COLUMN zero_trust_network_id UUID DEFAULT NULL REFERENCES zero_trust_networks (id) ON DELETE SET NULL;

