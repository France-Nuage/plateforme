import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.withSchema('billing').createTable('prices', (table) => {
      table.uuid('price__id', { primaryKey: true }).defaultTo(this.raw('uuid_generate_v4()'))
      table.string('name')
      table.string('resource_type')
      table.string('resource_unit')
      table.uuid('zone__id')
      table.string('pricing_unit')
      table.decimal('price_per_unit', 10, 2)
      table.datetime('effective_start')
      table.datetime('effective_end')
      table.timestamp('created_at', { useTz: true })
      table.timestamp('updated_at', { useTz: true })

      table.foreign('zone__id').references('zone__id').inTable('infrastructure.zones')
    })
  }
}
