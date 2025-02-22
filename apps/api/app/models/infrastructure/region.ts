import { BaseModel, belongsTo, column, hasMany } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo, HasMany } from '@adonisjs/lucid/types/relations'
import Zone from '#models/infrastructure/zone'
import Country from '#models/localisation/country'

export default class Region extends BaseModel {
  public static table = 'infrastructure.regions'

  @column({ isPrimary: true, columnName: 'region__id' })
  declare id: string

  @column()
  declare name: string

  @column({ columnName: 'country__id' })
  declare countryId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @hasMany(() => Zone)
  declare zones: HasMany<typeof Zone>

  @belongsTo(() => Country, { localKey: 'id', foreignKey: 'countryId' })
  declare country: BelongsTo<typeof Country>
}

export enum RegionName {
  LoireAtlantique = 'Loire Atlantique',
  Vendee = 'Vendée',
}
