import { BaseModel, belongsTo, column, computed } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import Instance from '#models/infrastructure/instance'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'

export default class InstanceTemplate extends BaseModel {
  public static table = 'infrastructure.instance_template'

  @computed()
  public get object() {
    return 'istance_template'
  }

  @column({ isPrimary: true, columnName: 'instance_template__id' })
  declare id: string

  @column()
  declare name: string

  @column()
  declare vcpus_min: number

  @column()
  declare vcpus_max: number

  @column()
  declare memory_gb: number

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Instance)
  declare instance: BelongsTo<typeof Instance>
}
