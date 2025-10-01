-- Modify "users" table
ALTER TABLE "public"."users" DROP COLUMN "organization_id";
-- Create "organization_user" table
CREATE TABLE "public"."organization_user" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "user_id" uuid NOT NULL,
  "organization_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "user_organizations_organization_id_fkey" FOREIGN KEY ("organization_id") REFERENCES "public"."organizations" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "user_organizations_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "user_organizations_user_id_organization_id_idx" to table: "organization_user"
CREATE UNIQUE INDEX "user_organizations_user_id_organization_id_idx" ON "public"."organization_user" ("user_id", "organization_id");
