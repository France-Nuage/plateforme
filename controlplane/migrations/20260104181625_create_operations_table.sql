-- Create "operations" table
CREATE TABLE "public"."operations" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "kind" text NOT NULL,
  "payload" jsonb NOT NULL,
  "status" text NOT NULL DEFAULT 'pending',
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "completed_at" timestamptz NULL,
  PRIMARY KEY ("id")
);
-- Drop "relationship_queue" table
DROP TABLE "public"."relationship_queue";
