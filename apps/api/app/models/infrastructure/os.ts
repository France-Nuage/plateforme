import {BaseModel, column, hasMany} from '@adonisjs/lucid/orm'
import {DateTime} from 'luxon'
import type {HasMany} from '@adonisjs/lucid/types/relations'
import OsVersion from '#models/infrastructure/os_version'

export default class Os extends BaseModel {
  public static table = 'os'

  @column({ columnName: 'os__id', isPrimary: true })
  declare id: string

  @column()
  declare name: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @hasMany(() => OsVersion)
  declare versions: HasMany<typeof OsVersion>
}
