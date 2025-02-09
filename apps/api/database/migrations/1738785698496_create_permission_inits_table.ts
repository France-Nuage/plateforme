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

    this.defer(async (db) => {
      await db.from('iam.role__permission').delete()
      await db.from('iam.permissions').delete()
      await db.from('iam.verbs').delete()
      await db.from('catalog.services').delete()
      await db.from('iam.types').delete()
      await db.from('iam.roles').delete()
    })

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
