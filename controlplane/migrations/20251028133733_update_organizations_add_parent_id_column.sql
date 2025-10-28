-- Modify "organizations" table
ALTER TABLE "public"."organizations" ADD COLUMN "parent_id" uuid NULL, ADD CONSTRAINT "organizations_parent_id_fkey" FOREIGN KEY ("parent_id") REFERENCES "public"."organizations" ("id") ON UPDATE NO ACTION ON DELETE SET NULL;
