-- Create "relationship_queue" table
CREATE TABLE "public"."relationship_queue" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "object_id" uuid NOT NULL,
  "object_type" text NOT NULL,
  "relation" text NOT NULL,
  "subject_id" uuid NOT NULL,
  "subject_type" text NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now()
);
