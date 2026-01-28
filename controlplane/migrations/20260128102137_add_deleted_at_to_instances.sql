-- Modify "instances" table
ALTER TABLE "public"."instances" ADD COLUMN "deleted_at" timestamptz NULL;
