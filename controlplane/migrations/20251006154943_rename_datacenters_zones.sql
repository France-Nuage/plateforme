-- Create "zones" table
CREATE TABLE "public"."zones" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "name" text NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id")
);
-- Rename a column from "datacenter_id" to "zone_id"
ALTER TABLE "public"."hypervisors" RENAME COLUMN "datacenter_id" TO "zone_id";
-- Modify "hypervisors" table
ALTER TABLE "public"."hypervisors" DROP CONSTRAINT "hypervisors_datacenter_id_fkey", ADD CONSTRAINT "hypervisors_zone_id_fkey" FOREIGN KEY ("zone_id") REFERENCES "public"."zones" ("id") ON UPDATE NO ACTION ON DELETE CASCADE;
-- Drop "datacenters" table
DROP TABLE "public"."datacenters";
