import {BaseModel, belongsTo, column} from '@adonisjs/lucid/orm'
import {DateTime} from 'luxon'
import Zone from '#models/infrastructure/zone'
import type {BelongsTo} from '@adonisjs/lucid/types/relations'

export default class Price extends BaseModel {
  public static table = 'billing.prices'

  @column({ isPrimary: true, columnName: 'price__id' })
  declare id: string

  @column()
  declare name: string

  @column()
  declare pricePerUnit: number

  @column()
  declare resourceUnit: string

  @column()
  declare resourceType: string

  @column()
  declare pricingUnit: string

  @column.dateTime()
  declare effective_start: DateTime

  @column.dateTime()
  declare effective_end: DateTime

  @column({ columnName: 'zone__id' })
  declare zoneId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Zone, { localKey: 'id', foreignKey: 'zone__id' })
  declare zone: BelongsTo<typeof Zone>
}
