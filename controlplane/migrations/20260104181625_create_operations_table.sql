-- Create "operations" table
-- atlas:nolint PG110
CREATE TABLE "public"."operations" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "kind" text NOT NULL,
  "payload" jsonb NOT NULL,
  "status" text NOT NULL DEFAULT 'pending',
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "completed_at" timestamptz NULL,
  PRIMARY KEY ("id")
);
-- Create indexes for efficient queue queries
CREATE INDEX "idx_operations_status" ON "public"."operations" ("status");
CREATE INDEX "idx_operations_status_created_at" ON "public"."operations" ("status", "created_at");
-- Drop "relationship_queue" table
-- atlas:nolint DS102
DROP TABLE "public"."relationship_queue";
