import { BaseModel, belongsTo, column, hasMany } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import Cluster from '#models/infrastructure/cluster'
import type { BelongsTo, HasMany } from '@adonisjs/lucid/types/relations'
import Instance from '#models/infrastructure/instance'

export default class Node extends BaseModel {
  public static table = 'infrastructure.nodes'

  @column({ isPrimary: true, columnName: 'node__id' })
  declare id: string

  @column()
  declare token: string

  @column()
  declare url: string

  @column()
  declare name: string

  @column({ columnName: 'cluster__id' })
  declare clusterId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoUpdate: true })
  declare updatedAt: DateTime

  @hasMany(() => Instance)
  declare instances: HasMany<typeof Instance>

  @belongsTo(() => Cluster, { localKey: 'id', foreignKey: 'clusterId' })
  declare cluster: BelongsTo<typeof Cluster>
}
