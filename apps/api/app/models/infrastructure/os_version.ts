import {BaseModel, belongsTo, column, computed} from '@adonisjs/lucid/orm'
import {DateTime} from 'luxon'
import type {BelongsTo} from '@adonisjs/lucid/types/relations'
import Os from '#models/infrastructure/os'

export default class OsVersion extends BaseModel {
  public static table = 'infrastructure.os_versions'

  @computed()
  public get object() {
    return 'os_version'
  }

  @column({ isPrimary: true, columnName: 'os_version__id' })
  declare id: string

  @column()
  declare os_name: string

  @column()
  declare version: string

  @column({ columnName: 'os__id' })
  declare osId: string

  @column()
  declare release_date: DateTime

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Os, { foreignKey: 'os__id', localKey: 'id' })
  declare os: BelongsTo<typeof Os>
}
