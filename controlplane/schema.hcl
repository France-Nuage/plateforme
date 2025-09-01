table "_sqlx_migrations" {
  schema = schema.public
  column "version" {
    null = false
    type = bigint
  }
  column "description" {
    null = false
    type = text
  }
  column "installed_on" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  column "success" {
    null = false
    type = boolean
  }
  column "checksum" {
    null = false
    type = bytea
  }
  column "execution_time" {
    null = false
    type = bigint
  }
  primary_key {
    columns = [column.version]
  }
}
table "datacenters" {
  schema = schema.public
  column "id" {
    null    = false
    type    = uuid
    default = sql("gen_random_uuid()")
  }
  column "name" {
    null = false
    type = text
  }
  column "created_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  column "updated_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  primary_key {
    columns = [column.id]
  }
}
table "hypervisors" {
  schema = schema.public
  column "id" {
    null    = false
    type    = uuid
    default = sql("gen_random_uuid()")
  }
  column "url" {
    null = false
    type = text
  }
  column "authorization_token" {
    null = false
    type = text
  }
  column "storage_name" {
    null = false
    type = text
  }
  column "organization_id" {
    null = false
    type = uuid
  }
  column "datacenter_id" {
    null = false
    type = uuid
  }
  primary_key {
    columns = [column.id]
  }
  foreign_key "hypervisors_datacenter_id_fkey" {
    columns     = [column.datacenter_id]
    ref_columns = [table.datacenters.column.id]
    on_update   = NO_ACTION
    on_delete   = CASCADE
  }
  foreign_key "hypervisors_organization_id_fkey" {
    columns     = [column.organization_id]
    ref_columns = [table.organizations.column.id]
    on_update   = NO_ACTION
    on_delete   = CASCADE
  }
}
table "instances" {
  schema = schema.public
  column "id" {
    null    = false
    type    = uuid
    default = sql("gen_random_uuid()")
  }
  column "hypervisor_id" {
    null = false
    type = uuid
  }
  column "distant_id" {
    null = false
    type = text
  }
  column "status" {
    null    = false
    type    = character_varying(50)
    default = "UNKNOWN"
  }
  column "max_cpu_cores" {
    null    = false
    type    = integer
    default = 1
  }
  column "cpu_usage_percent" {
    null    = false
    type    = double_precision
    default = 0
  }
  column "max_memory_bytes" {
    null    = false
    type    = bigint
    default = 1073741824
  }
  column "memory_usage_bytes" {
    null    = false
    type    = bigint
    default = 0
  }
  column "name" {
    null    = false
    type    = character_varying(255)
    default = ""
  }
  column "created_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  column "updated_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  column "project_id" {
    null = false
    type = uuid
  }
  column "max_disk_bytes" {
    null    = false
    type    = bigint
    default = 1073741824
  }
  column "disk_usage_bytes" {
    null    = false
    type    = bigint
    default = 0
  }
  column "ip_v4" {
    null    = false
    type    = character_varying(255)
    default = "0.0.0.0"
  }
  column "zero_trust_network_id" {
    null = true
    type = uuid
  }
  primary_key {
    columns = [column.id]
  }
  foreign_key "instances_hypervisor_id_fkey" {
    columns     = [column.hypervisor_id]
    ref_columns = [table.hypervisors.column.id]
    on_update   = NO_ACTION
    on_delete   = CASCADE
  }
  foreign_key "instances_project_id_fkey" {
    columns     = [column.project_id]
    ref_columns = [table.projects.column.id]
    on_update   = NO_ACTION
    on_delete   = CASCADE
  }
  foreign_key "instances_zero_trust_network_id_fkey" {
    columns     = [column.zero_trust_network_id]
    ref_columns = [table.zero_trust_networks.column.id]
    on_update   = NO_ACTION
    on_delete   = SET_NULL
  }
  index "idx_instances_hypervisor_id" {
    columns = [column.hypervisor_id]
  }
}
table "organizations" {
  schema = schema.public
  column "id" {
    null    = false
    type    = uuid
    default = sql("gen_random_uuid()")
  }
  column "name" {
    null = false
    type = text
  }
  column "created_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  column "updated_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  primary_key {
    columns = [column.id]
  }
}
table "projects" {
  schema = schema.public
  column "id" {
    null    = false
    type    = uuid
    default = sql("gen_random_uuid()")
  }
  column "name" {
    null = false
    type = text
  }
  column "organization_id" {
    null = false
    type = uuid
  }
  column "created_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  column "updated_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  primary_key {
    columns = [column.id]
  }
  foreign_key "projects_organization_id_fkey" {
    columns     = [column.organization_id]
    ref_columns = [table.organizations.column.id]
    on_update   = NO_ACTION
    on_delete   = CASCADE
  }
}
table "zero_trust_network_types" {
  schema = schema.public
  column "id" {
    null    = false
    type    = uuid
    default = sql("gen_random_uuid()")
  }
  column "name" {
    null = false
    type = text
  }
  column "created_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  column "updated_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  primary_key {
    columns = [column.id]
  }
}
table "zero_trust_networks" {
  schema = schema.public
  column "id" {
    null    = false
    type    = uuid
    default = sql("gen_random_uuid()")
  }
  column "name" {
    null = false
    type = text
  }
  column "organization_id" {
    null = false
    type = uuid
  }
  column "zero_trust_network_type_id" {
    null = false
    type = uuid
  }
  column "created_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  column "updated_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
  }
  primary_key {
    columns = [column.id]
  }
  foreign_key "zero_trust_networks_organization_id_fkey" {
    columns     = [column.organization_id]
    ref_columns = [table.organizations.column.id]
    on_update   = NO_ACTION
    on_delete   = CASCADE
  }
  foreign_key "zero_trust_networks_zero_trust_network_type_id_fkey" {
    columns     = [column.zero_trust_network_type_id]
    ref_columns = [table.zero_trust_network_types.column.id]
    on_update   = NO_ACTION
    on_delete   = CASCADE
  }
}
schema "public" {
  comment = "standard public schema"
}
