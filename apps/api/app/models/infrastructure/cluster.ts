import { BaseModel, belongsTo, column, hasMany } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo, HasMany } from '@adonisjs/lucid/types/relations'
import Instance from '#models/infrastructure/instance'
import Zone from '#models/infrastructure/zone'

export default class Cluster extends BaseModel {
  public static table = 'infrastructure.clusters'

  @column({ isPrimary: true, columnName: 'cluster__id' })
  declare id: string

  @column({ columnName: 'zone__id' })
  declare zoneId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @hasMany(() => Instance)
  declare instances: HasMany<typeof Instance>

  @belongsTo(() => Zone, { localKey: 'id', foreignKey: 'zoneId' })
  declare zone: BelongsTo<typeof Zone>
}
