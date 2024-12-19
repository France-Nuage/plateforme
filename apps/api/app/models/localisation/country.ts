import { BaseModel, belongsTo, column } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'
import { Region } from '#models/infrastructure/region'

export class Country extends BaseModel {
  public static table = 'localisation.countries'

  @column({ isPrimary: true, columnName: 'country__id' })
  declare id: string

  @column()
  declare name: string

  @column()
  declare code: string

  @column()
  declare latitude: number

  @column()
  declare longitude: number

  @column()
  declare phoneIndicator: string

  @column()
  declare phoneRegex: string

  @column()
  declare postalCodeRegex: string

  @column()
  declare flagSvg: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Region)
  declare regions: BelongsTo<typeof Region>
}
