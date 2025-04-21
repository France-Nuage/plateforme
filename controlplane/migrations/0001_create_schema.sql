CREATE TABLE hypervisors (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    url TEXT NOT NULL,
    authorization_token TEXT NOT NULL,
    storage_name TEXT NOT NULL
);

CREATE TABLE instances (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    hypervisor_id UUID NOT NULL REFERENCES hypervisors (id) ON DELETE CASCADE,
    distant_id TEXT NOT NULL
);

CREATE INDEX idx_instances_hypervisor_id ON instances (hypervisor_id);
