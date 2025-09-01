-- Create "_sqlx_migrations" table
CREATE TABLE "public"."_sqlx_migrations" (
  "version" bigint NOT NULL,
  "description" text NOT NULL,
  "installed_on" timestamptz NOT NULL DEFAULT now(),
  "success" boolean NOT NULL,
  "checksum" bytea NOT NULL,
  "execution_time" bigint NOT NULL,
  PRIMARY KEY ("version")
);
-- Create "datacenters" table
CREATE TABLE "public"."datacenters" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "name" text NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id")
);
-- Create "organizations" table
CREATE TABLE "public"."organizations" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "name" text NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id")
);
-- Create "hypervisors" table
CREATE TABLE "public"."hypervisors" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "url" text NOT NULL,
  "authorization_token" text NOT NULL,
  "storage_name" text NOT NULL,
  "organization_id" uuid NOT NULL,
  "datacenter_id" uuid NOT NULL,
  PRIMARY KEY ("id"),
  CONSTRAINT "hypervisors_datacenter_id_fkey" FOREIGN KEY ("datacenter_id") REFERENCES "public"."datacenters" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "hypervisors_organization_id_fkey" FOREIGN KEY ("organization_id") REFERENCES "public"."organizations" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "projects" table
CREATE TABLE "public"."projects" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "name" text NOT NULL,
  "organization_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "projects_organization_id_fkey" FOREIGN KEY ("organization_id") REFERENCES "public"."organizations" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "zero_trust_network_types" table
CREATE TABLE "public"."zero_trust_network_types" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "name" text NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id")
);
-- Create "zero_trust_networks" table
CREATE TABLE "public"."zero_trust_networks" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "name" text NOT NULL,
  "organization_id" uuid NOT NULL,
  "zero_trust_network_type_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id"),
  CONSTRAINT "zero_trust_networks_organization_id_fkey" FOREIGN KEY ("organization_id") REFERENCES "public"."organizations" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "zero_trust_networks_zero_trust_network_type_id_fkey" FOREIGN KEY ("zero_trust_network_type_id") REFERENCES "public"."zero_trust_network_types" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "instances" table
CREATE TABLE "public"."instances" (
  "id" uuid NOT NULL DEFAULT gen_random_uuid(),
  "hypervisor_id" uuid NOT NULL,
  "distant_id" text NOT NULL,
  "status" character varying(50) NOT NULL DEFAULT 'UNKNOWN',
  "max_cpu_cores" integer NOT NULL DEFAULT 1,
  "cpu_usage_percent" double precision NOT NULL DEFAULT 0,
  "max_memory_bytes" bigint NOT NULL DEFAULT 1073741824,
  "memory_usage_bytes" bigint NOT NULL DEFAULT 0,
  "name" character varying(255) NOT NULL DEFAULT '',
  "created_at" timestamptz NOT NULL DEFAULT now(),
  "updated_at" timestamptz NOT NULL DEFAULT now(),
  "project_id" uuid NOT NULL,
  "max_disk_bytes" bigint NOT NULL DEFAULT 1073741824,
  "disk_usage_bytes" bigint NOT NULL DEFAULT 0,
  "ip_v4" character varying(255) NOT NULL DEFAULT '0.0.0.0',
  "zero_trust_network_id" uuid NULL,
  PRIMARY KEY ("id"),
  CONSTRAINT "instances_hypervisor_id_fkey" FOREIGN KEY ("hypervisor_id") REFERENCES "public"."hypervisors" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "instances_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "public"."projects" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "instances_zero_trust_network_id_fkey" FOREIGN KEY ("zero_trust_network_id") REFERENCES "public"."zero_trust_networks" ("id") ON UPDATE NO ACTION ON DELETE SET NULL
);
-- Create index "idx_instances_hypervisor_id" to table: "instances"
CREATE INDEX "idx_instances_hypervisor_id" ON "public"."instances" ("hypervisor_id");
