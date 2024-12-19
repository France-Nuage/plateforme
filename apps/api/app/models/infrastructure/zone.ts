import { BaseModel, belongsTo, column, hasMany } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo, HasMany } from '@adonisjs/lucid/types/relations'
import Cluster from '#models/infrastructure/cluster'
import Region from '#models/infrastructure/region'

export default class Zone extends BaseModel {
  public static table = 'infrastructure.zones'

  @column({ isPrimary: true, columnName: 'zone__id' })
  declare id: string

  @column({ columnName: 'region__id' })
  declare regionId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @hasMany(() => Cluster)
  declare clusters: HasMany<typeof Cluster>

  @belongsTo(() => Region, { localKey: 'id', foreignKey: 'regionId' })
  declare region: BelongsTo<typeof Region>
}
