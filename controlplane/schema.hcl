table "zones" {
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
  column "zone_id" {
    null = false
    type = uuid
  }
  primary_key {
    columns = [column.id]
  }
  foreign_key "hypervisors_zone_id_fkey" {
    columns     = [column.zone_id]
    ref_columns = [table.zones.column.id]
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
table "organization_service_account" {
  schema = schema.public
    column "id" {
    null    = false
    type    = uuid
    default = sql("gen_random_uuid()")
  }
  column "service_account_id" {
    null = false
    type = uuid
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
  foreign_key "organization_service_account_service_account_id_fkey" {
    columns = [column.service_account_id]
    ref_columns = [table.service_accounts.column.id]
    on_update = NO_ACTION
    on_delete = CASCADE
  }
  foreign_key "organization_service_account_organization_id_fkey" {
    columns = [column.organization_id]
    ref_columns = [table.organizations.column.id]
    on_update = NO_ACTION
    on_delete = CASCADE
  }
  index "organization_service_account_service_account_id_organization_id_idx" {
    unique = true
    columns = [column.service_account_id, column.organization_id]
  }
}
table "organization_user" {
  schema = schema.public
    column "id" {
    null    = false
    type    = uuid
    default = sql("gen_random_uuid()")
  }
  column "user_id" {
    null = false
    type = uuid
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
  foreign_key "user_organizations_user_id_fkey" {
    columns = [column.user_id]
    ref_columns = [table.users.column.id]
    on_update = NO_ACTION
    on_delete = CASCADE
  }
  foreign_key "user_organizations_organization_id_fkey" {
    columns = [column.organization_id]
    ref_columns = [table.organizations.column.id]
    on_update = NO_ACTION
    on_delete = CASCADE
  }
  index "user_organizations_user_id_organization_id_idx" {
    unique = true
    columns = [column.user_id, column.organization_id]
  }
}
table "relationship_queue" {
  schema = schema.public
  column "id" {
    null = false
    type = uuid
    default = sql("gen_random_uuid()")
  }
  column "object_id" {
    null = false
    type = text
  }
  column "object_type" {
    null = false
    type = text
  }
  column "relation" {
    null = false
    type = text
  }
  column "subject_id" {
    null = false
    type= text
  }
  column "subject_type" {
    null = false
    type = text
  }
  column "created_at" {
    null    = false
    type    = timestamptz
    default = sql("now()")
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
table "service_accounts" {
  schema = schema.public
  column "id" {
    null = false
    type = uuid
    default = sql("gen_random_uuid()")
  }
  column "name" {
    null = false
    type = text
    default = false
  }
  column "key" {
    null = false
    type = text
    default = false
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
table "users" {
  schema = schema.public
  column "id" {
    null = false
    type = uuid
    default = sql("gen_random_uuid()")
  }
  column "email" {
    null = false
    type = text
  }
  column "is_admin" {
    null = false
    type = bool
    default = false
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
  index "users_email_idx" {
    unique = true
    columns = [column.email]
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
