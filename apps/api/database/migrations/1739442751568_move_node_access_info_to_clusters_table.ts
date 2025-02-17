import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.withSchema('infrastructure').table('clusters', (table) => {
      table.string('token_secret').nullable().after('name')
      table.string('token_id').nullable().after('name')
      table.string('host').nullable().after('name')
      table.uuid('organization__id').nullable().after('zone__id')

      table
        .foreign('organization__id')
        .references('organization__id')
        .inTable('resource.organizations')
    })

    this.schema.withSchema('infrastructure').table('nodes', (table) => {
      table.dropColumn('token')
      table.dropColumn('url')
    })
  }
}
