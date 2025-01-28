import {BaseSchema} from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.raw('CREATE EXTENSION IF NOT EXISTS "uuid-ossp" schema pg_catalog version "1.1";')

    this.schema.createSchema('stripe')
    this.schema.createSchema('resource')

    this.schema.withSchema('resource').createTable('organizations', (table) => {
      table.uuid('organization__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.string('email')
      table.string('fax')
      table.string('phone')
      table.string('establishment_identifier')
      table.integer('owner__id')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.foreign('owner__id').references('id').inTable('member.users')
    })

    this.schema.withSchema('resource').createTable('folders', (table) => {
      table.uuid('folder__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.uuid('organization__id')
      table.string('name')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table
        .foreign('organization__id')
        .references('organization__id')
        .inTable('resource.organizations')
    })

    this.schema.withSchema('resource').createTable('projects', (table) => {
      table.uuid('project__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.string('description')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.uuid('folder__id')
      table.foreign('folder__id').references('folder__id').inTable('resource.folders')
    })

    this.schema.createSchema('catalog')
    this.schema.withSchema('catalog').createTable('services', (table) => {
      table.string('service__id', 63).primary()
      table.string('description')
    })

    this.schema.withSchema('iam').createTable('tokens', (table) => {
      table.uuid('id')
      table.string('email').notNullable().index()
      table.string('token').notNullable().unique()
      table.timestamp('expires_at').notNullable()
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })
    })

    this.schema.withSchema('iam').createTable('types', (table) => {
      table.string('type__id', 63)
      table
        .string('service__id', 63)
        .references('service__id')
        .inTable('catalog.services')
        .onDelete('cascade')
        .onUpdate('cascade')
      table.string('description')

      table.primary(['type__id', 'service__id'])
    })

    this.schema.withSchema('iam').createTable('verbs', (table) => {
      table.string('verb__id', 63).primary()
    })

    this.schema.withSchema('iam').createTable('permissions', (table) => {
      table.uuid('permission__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('type__id', 63)
      table
        .string('service__id', 63)
        .references('service__id')
        .inTable('catalog.services')
        .onDelete('restrict')
        .onUpdate('cascade')
      table
        .string('verb__id', 63)
        .references('verb__id')
        .inTable('iam.verbs')
        .onDelete('restrict')
        .onDelete('cascade')
      table
        .foreign(['service__id', 'type__id'])
        .references(['service__id', 'type__id'])
        .inTable('iam.types')
        .onDelete('restrict')
        .onUpdate('cascade')

      table.unique(['service__id', 'type__id', 'verb__id'])
    })

    this.schema.withSchema('iam').createTable('roles', (table) => {
      table.string('role__id', 63)
      table
        .string('service__id')
        .references('service__id')
        .inTable('catalog.services')
        .onDelete('restrict')
        .onUpdate('cascade')
      table.string('description')
      table.string('title')

      table.primary(['role__id'])
      table.unique(['service__id', 'role__id'])
    })

    this.schema.withSchema('iam').createTable('role__permission', (table) => {
      table.string('role__id')
      table.uuid('permission__id')

      table
        .foreign('permission__id')
        .references('permission__id')
        .inTable('iam.permissions')
        .onDelete('restrict')
        .onUpdate('cascade')

      table
        .foreign('role__id')
        .references('role__id')
        .inTable('iam.roles')
        .onDelete('restrict')
        .onUpdate('cascade')

      table.unique(['role__id', 'permission__id'])
    })

    this.schema.withSchema('iam').createTable('resource_policy', (table) => {
      table.uuid('policy__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table
        .uuid('organization__id')
        .references('organization__id')
        .inTable('resource.organizations')
        .onDelete('cascade')
        .onUpdate('cascade')
      table
        .uuid('folder__id')
        .references('folder__id')
        .inTable('resource.folders')
        .onDelete('cascade')
        .onUpdate('cascade')
      table
        .uuid('project__id')
        .references('project__id')
        .inTable('resource.projects')
        .onDelete('cascade')
        .onUpdate('cascade')

      table.unique(['organization__id', 'folder__id', 'project__id'])
    })

    this.schema.withSchema('iam').createTable('user_resource_policy_binding', (table) => {
      table.uuid('binding__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table
        .uuid('policy__id')
        .references('policy__id')
        .inTable('iam.resource_policy')
        .onDelete('cascade')
        .onUpdate('cascade')
      table
        .integer('member__id')
        .references('id')
        .inTable('member.users')
        .onDelete('cascade')
        .onUpdate('cascade')
      table.string('role__id')
      table.string('service__id')

      table
        .foreign(['service__id', 'role__id'])
        .references(['service__id', 'role__id'])
        .inTable('iam.roles')
        .onDelete('cascade')
        .onUpdate('cascade')

      table.unique(['policy__id', 'member__id'])
    })

    // this.schema.createSchema('service')
    // this.schema.withSchema('service').createTable('services', (table) => {
    //   table.uuid('service__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
    //   table.string('name')
    //   table.timestamp('created_at', { useTz: true })
    //   table.timestamp('updated_at', { useTz: true })
    // })

    // this.schema.withSchema('service').createTable('versions', (table) => {
    //   table.uuid('version__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
    //   table.string('name')
    //   table.string('description')
    //   table.timestamp('available_at')
    //   table.timestamp('created_at', { useTz: true })
    //   table.timestamp('updated_at', { useTz: true })
    //
    //   table.uuid('service__id')
    //   table.foreign('service__id').references('service__id').inTable('service.services')
    // })

    this.schema.createSchema('localisation')
    this.schema.withSchema('localisation').createTable('countries', (table) => {
      table.uuid('country__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.string('code')
      table.float('latitude')
      table.float('longitude')
      table.string('postal_code_regex')
      table.string('phone_indicator')
      table.string('phone_regex')
      table.text('flag_svg')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })
    })
  }

  async down() {
    this.schema.withSchema('iam').dropTable('roles')
    this.schema.withSchema('iam').dropTable('permissions')
    this.schema.withSchema('iam').dropTable('verbs')
    this.schema.withSchema('iam').dropTable('types')
    this.schema.withSchema('iam').dropTable('services')

    this.schema.withSchema('resources').dropTable('projects')
    this.schema.withSchema('resources').dropTable('organizations')
    this.schema.withSchema('resources').dropTable('folders')
    this.schema.dropSchema('resources')

    this.schema.withSchema('service').dropTable('services')
    this.schema.withSchema('service').dropTable('versions')
    this.schema.dropSchema('service')

    this.schema.withSchema('localisation').dropTable('countries')
    this.schema.dropSchema('localisation')

    this.schema.dropSchema('stripe')
  }
}
