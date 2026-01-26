-- Add soft delete column to instances table
ALTER TABLE "public"."instances" ADD COLUMN "deleted_at" timestamptz NULL;

-- Create index on deleted_at for efficient filtering
CREATE INDEX "idx_instances_deleted_at" ON "public"."instances" ("deleted_at");

-- Create composite index for common queries (hypervisor_id + deleted_at)
CREATE INDEX "idx_instances_hypervisor_id_deleted_at" ON "public"."instances" ("hypervisor_id", "deleted_at");
