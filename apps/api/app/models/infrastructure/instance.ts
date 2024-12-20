import { BaseModel, belongsTo, column } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'
import BootDisk from "#models/infrastructure/boot_disk";

export default class Instance extends BaseModel {
  public static table = 'infrastructure.instances'

  @column({ isPrimary: true, columnName: 'instance__id' })
  declare id: string

  @column({ columnName: 'cluster__id' })
  declare clusterId: string

  @column({ columnName: 'boot_disk__id' })
  declare bootDiskId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Instance, { localKey: 'id', foreignKey: 'clusterId' })
  declare cluster: BelongsTo<typeof Instance>

  @belongsTo(() => BootDisk, { foreignKey: 'bootDiskId', localKey: 'id' })
  declare bootDisk: BelongsTo<typeof BootDisk>
}
