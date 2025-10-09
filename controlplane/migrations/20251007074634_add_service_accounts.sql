-- Create "service_accounts" table
CREATE TABLE "public"."service_accounts" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "name" text NOT NULL DEFAULT 'false',
  "key" text NOT NULL DEFAULT 'false',
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id")
);
-- Create "organization_service_account" table
CREATE TABLE "public"."organization_service_account" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "service_account_id" uuid NOT NULL,
  "organization_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "organization_service_account_organization_id_fkey" FOREIGN KEY ("organization_id") REFERENCES "public"."organizations" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "organization_service_account_service_account_id_fkey" FOREIGN KEY ("service_account_id") REFERENCES "public"."service_accounts" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "organization_service_account_service_account_id_organization_id" to table: "organization_service_account"
CREATE UNIQUE INDEX "organization_service_account_service_account_id_organization_id" ON "public"."organization_service_account" ("service_account_id", "organization_id");
