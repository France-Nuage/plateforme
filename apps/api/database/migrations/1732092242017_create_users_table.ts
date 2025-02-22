import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  protected tableName = 'users'

  async up() {
    this.schema.createSchema('member')
    this.schema.withSchema('member').createTable(this.tableName, (table) => {
      table.increments('id').notNullable()
      table.string('lastname')
      table.string('firstname')
      table.string('email', 254).notNullable().unique()
      table.string('password').notNullable()

      table.timestamp('created_at').notNullable()
      table.timestamp('updated_at').nullable()
    })
  }

  async down() {
    this.schema.dropSchema('billing', true)
    this.schema.dropSchema('catalog', true)
    this.schema.dropSchema('iam', true)
    this.schema.dropSchema('infrastructure', true)
    this.schema.dropSchema('localisation', true)
    this.schema.dropSchema('member', true)
    this.schema.dropSchema('quota', true)
    this.schema.dropSchema('resource', true)
    this.schema.dropSchema('stripe', true)
  }
}
