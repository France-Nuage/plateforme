import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.createSchema('infrastructure')
    this.schema.withSchema('infrastructure').createTable('regions', (table) => {
      table.uuid('region__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.uuid('country__id')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.foreign('country__id').references('country__id').inTable('localisation.countries')
    })

    this.schema.withSchema('infrastructure').createTable('zones', (table) => {
      table.uuid('zone__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.uuid('region__id')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.foreign('region__id').references('region__id').inTable('infrastructure.regions')
    })

    this.schema.withSchema('infrastructure').createTable('clusters', (table) => {
      table.uuid('cluster__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.uuid('zone__id')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.foreign('zone__id').references('zone__id').inTable('infrastructure.zones')
    })

    this.schema.withSchema('infrastructure').createTable('nodes', (table) => {
      table.uuid('node__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('url')
      table.string('token')
      table.uuid('cluster__id')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.foreign('cluster__id').references('cluster__id').inTable('infrastructure.clusters')
    })

    this.schema.withSchema('infrastructure').createTable('instance_types', (table) => {
      table
        .uuid('instance_type__id', { primaryKey: true })
        .defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name').notNullable()
      // table.string('series').notNullable()
      table.string('description').notNullable()
      table.integer('vcpus_min').notNullable()
      table.integer('vcpus_max').notNullable()
      table.integer('memory_min_gb').notNullable()
      table.integer('memory_max_gb').notNullable()
      table.string('platform').notNullable()
      table.decimal('estimated_monthly_cost', 10, 2)
      table.timestamps(true)
    })

    this.schema
      .withSchema('infrastructure')
      .createTable('instance_template_categories', (table) => {
        table
          .uuid('instance_template_category__id', { primaryKey: true })
          .defaultTo(this.raw('uuid_generate_v4()'))
        table.string('name').notNullable()
        table.uuid('instance_type__id')

        table
          .foreign('instance_type__id')
          .references('instance_type__id')
          .inTable('infrastructure.instance_types')
      })

    this.schema.withSchema('infrastructure').createTable('instance_template', (table) => {
      table
        .uuid('instance_template__id', { primaryKey: true })
        .defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name').notNullable()
      table.integer('vcpus_min').notNullable()
      table.integer('vcpus_max').notNullable()
      table.integer('memory_gb').notNullable()
      table.integer('memory_max_gb').notNullable()
      table.timestamps(true)

      table.uuid('instance_template_category__id')
      table
        .foreign('instance_template_category__id')
        .references('instance_template_category__id')
        .inTable('infrastructure.instance_template_categories')
    })

    this.schema.withSchema('infrastructure').createTable('instance_types__zones', (table) => {
      table.uuid('instance_type__id')
      table.uuid('zone__id')
      table.boolean('publicly_available')

      table.foreign('zone__id').references('zone__id').inTable('infrastructure.zones')
      table
        .foreign('instance_type__id')
        .references('instance_type__id')
        .inTable('infrastructure.instance_types')
    })

    this.schema.withSchema('infrastructure').createTable('pricing_versions', (table) => {
      table
        .uuid('pricing_version__id', { primaryKey: true })
        .defaultTo(this.raw('uuid_generate_v4()'))
      table.decimal('price_per_vcpu', 10, 2).notNullable()
      table.decimal('price_per_gb_ram', 10, 2).notNullable()
      table.string('region').notNullable()
      table.date('effective_date').notNullable()
      table.timestamps(true)
    })

    this.schema.withSchema('infrastructure').createTable('boot_disks', (table) => {
      table.uuid('boot_disk__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('os').notNullable()
      table.string('disk_type').notNullable()
      table.integer('size_gb').notNullable()
      table.timestamps(true)
    })

    this.schema.withSchema('infrastructure').createTable('os', (table) => {
      table.uuid('os__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
    })

    this.schema.withSchema('infrastructure').createTable('os_versions', (table) => {
      table.uuid('id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('os_name').notNullable()
      table.uuid('os__id').notNullable()
      table.string('version').notNullable()
      table.date('release_date')
      table.timestamps(true)
      table.uuid('boot_disk__id').notNullable()
      table.foreign('os__id').references('os__id').inTable('infrastructure.os')
    })

    this.schema.withSchema('infrastructure').createTable('instances', (table) => {
      table.uuid('instance__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.string('node')
      table.string('pve_vm_id')
      table.uuid('node__id')
      table.uuid('project__id').notNullable()
      table.uuid('instance_type__id')
      table.uuid('boot_disk__id').notNullable()
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.foreign('project__id').references('project__id').inTable('resource.projects')
      table.foreign('node__id').references('node__id').inTable('infrastructure.nodes')
      table
        .foreign('instance_type__id')
        .references('instance_type__id')
        .inTable('infrastructure.instance_types')
      table
        .foreign('boot_disk__id')
        .references('boot_disk__id')
        .inTable('infrastructure.boot_disks')
    })
  }

  async down() {
    this.schema.withSchema('infrastructure').dropTable('instances')
    this.schema.withSchema('infrastructure').dropTable('clusters')
    this.schema.withSchema('infrastructure').dropTable('zones')
    this.schema.withSchema('infrastructure').dropTable('regions')
    this.schema.dropSchema('infrastructure')

    this.schema.dropSchema('stripe')
  }
}
