import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.withSchema('iam').table('role__permission', (table) => {
      table.dropForeign('permission__id')
      table.dropColumn('permission__id')
    })

    this.schema.withSchema('iam').table('permissions', (table) => {
      table.dropPrimary('permissions_pkey')
      table.dropColumn('permission__id')
    })

    this.db.from('iam.role__permission').delete()
    this.db.from('iam.permissions').delete()
    this.db.from('iam.verbs').delete()
    this.db.from('catalog.services').delete()
    this.db.from('iam.types').delete()

    this.schema.withSchema('iam').table('permissions', (table) => {
      table.string('permission__id', 255)

      table.primary(['permission__id'])
    })

    this.schema.withSchema('iam').alterTable('role__permission', (table) => {
      table.string('permission__id', 255)
      table
        .foreign('permission__id')
        .references('permission__id')
        .inTable('iam.permissions')
        .onDelete('restrict')
        .onUpdate('cascade')

      table.unique(['role__id', 'permission__id'])
    })
  }
}
