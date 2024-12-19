import { BaseModel, belongsTo, column } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'
import Zone from '#models/infrastructure/zone'
import Country from '#models/localisation/country'

export default class Region extends BaseModel {
  public static table = 'infrastructure.regions'

  @column({ isPrimary: true, columnName: 'region__id' })
  declare id: string

  @column({ columnName: 'country__id' })
  declare countryId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Zone)
  declare zones: BelongsTo<typeof Zone>

  @belongsTo(() => Country, { localKey: 'id', foreignKey: 'countryId' })
  declare country: BelongsTo<typeof Country>
}
