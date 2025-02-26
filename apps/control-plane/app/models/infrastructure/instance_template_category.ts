import { BaseModel, column, computed, hasMany } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import InstanceTemplate from '#models/infrastructure/instance_template'
import type { HasMany } from '@adonisjs/lucid/types/relations'

export default class InstanceTemplateCategory extends BaseModel {
  public static table = 'infrastructure.instance_template_category'

  @computed()
  public get object() {
    return 'instance_template_category'
  }

  @column({ isPrimary: true, columnName: 'instance_template_category__id' })
  declare id: string

  @column()
  declare name: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @hasMany(() => InstanceTemplate)
  declare templates: HasMany<typeof InstanceTemplate>
}
