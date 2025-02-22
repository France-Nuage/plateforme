import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.withSchema('infrastructure').table('nodes', (table) => {
      table.string('name').after('node__id')
    })
  }
}
