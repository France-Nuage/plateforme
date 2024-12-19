import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.createSchema('billing')
    this.schema.withSchema('billing').createTable('accounts', (table) => {
      table.uuid('account__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.boolean('open')
      table.string('displayName').notNullable()
      table.string('currencyCode').notNullable().comment('cannot be updated')
      table.uuid('folder__id')

      table.string('payment_profile__id', 13)

      table
        .foreign('folder__id')
        .references('folder__id')
        .inTable('resource.folders')
        .onDelete('cascade')
        .onDelete('cascade')
    })

    this.schema.withSchema('resource').alterTable('folders', (table) => {
      table.uuid('account__id')

      table.foreign('account__id').references('account__id').inTable('billing.accounts')
    })
  }

  async down() {
    this.schema.withSchema('billing').dropTable('accounts')
  }
}
