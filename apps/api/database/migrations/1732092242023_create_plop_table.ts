import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.withSchema('billing').createTable('test', (table) => {
      table.uuid('test__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

    })
  }

  async down() {
    this.schema.withSchema('billing').dropTable('test')
  }
}
