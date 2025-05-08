ALTER TABLE instances
ADD COLUMN status VARCHAR(50) NOT NULL DEFAULT 'UNKNOWN' CHECK (status IN ('UNKNOWN', 'RUNNING', 'STOPPED', 'FAILED', 'STARTING', 'STOPPING')),
ADD COLUMN max_cpu_cores INTEGER NOT NULL DEFAULT 1 CHECK (max_cpu_cores > 0 AND max_cpu_cores <= 99),
ADD COLUMN cpu_usage_percent FLOAT NOT NULL DEFAULT 0.0 CHECK (cpu_usage_percent >= 0.0 AND cpu_usage_percent <= 100.0),
ADD COLUMN max_memory_bytes BIGINT NOT NULL DEFAULT 1073741824 CHECK (max_memory_bytes > 0 AND max_memory_bytes <= 68719476736),
ADD COLUMN memory_usage_bytes BIGINT NOT NULL DEFAULT 0 CHECK (memory_usage_bytes >= 0),
ADD COLUMN name VARCHAR(255) NOT NULL DEFAULT '';

-- Add constraint to ensure memory_usage_bytes cannot exceed max_memory_bytes
ALTER TABLE instances
ADD CONSTRAINT check_memory_usage CHECK (memory_usage_bytes <= max_memory_bytes);
