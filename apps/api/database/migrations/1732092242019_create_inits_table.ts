import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  protected tableName = 'inits'

  async up() {
    this.schema.raw('CREATE EXTENSION IF NOT EXISTS "uuid-ossp" schema pg_catalog version "1.1";')

    this.schema.createSchema('iam')
    this.schema.withSchema('iam').createTable('environments', (table) => {
      table.uuid('environment__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })
    })

    this.schema.withSchema('iam').createTable('organizations', (table) => {
      table.uuid('organization__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.string('email')
      table.string('fax')
      table.string('phone')
      table.string('establishment_identifier')
      table.uuid('environment__id')
      table.integer('owner__id')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.foreign('owner__id').references('id').inTable('users')
      table.foreign('environment__id').references('environment__id').inTable('iam.environments')
    })

    this.schema.withSchema('iam').createTable('projects', (table) => {
      table.uuid('project__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.string('description')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.uuid('organization__id')
      table.foreign('organization__id').references('organization__id').inTable('iam.organizations')
    })

    this.schema.withSchema('iam').createTable('roles', (table) => {
      table.uuid('role__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.uuid('organization__id')
      table.foreign('organization__id').references('organization__id').inTable('iam.organizations')
    })

    this.schema.createSchema('service')
    this.schema.withSchema('service').createTable('services', (table) => {
      table.uuid('service__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })
    })

    this.schema.withSchema('service').createTable('versions', (table) => {
      table.uuid('version__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.string('description')
      table.timestamp('available_at')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.uuid('service__id')
      table
        .foreign('service__id')
        .references('service__id')
        .inTable('service.services')
    })
  }

  async down() {
    this.schema.withSchema('iam').dropTable('projects')
    this.schema.withSchema('iam').dropTable('organizations')
    this.schema.withSchema('iam').dropTable('environments')
    this.schema.dropSchema('iam')

    this.schema.withSchema('application').dropTable('applications')
    this.schema.withSchema('application').dropTable('versions')
    this.schema.dropSchema('application')
  }
}
