import { BaseModel, belongsTo, column } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'
import BootDisk from '#models/infrastructure/boot_disk'
import Node from '#models/infrastructure/node'
import Project from '#models/resource/project'

export default class Instance extends BaseModel {
  public static table = 'infrastructure.instances'

  @column({ isPrimary: true, columnName: 'instance__id' })
  declare id: string

  @column()
  declare pveVmId: string

  @column()
  declare name: string

  @column({ columnName: 'project__id' })
  declare projectId: string

  @column({ columnName: 'node__id' })
  declare nodeId: string

  @column({ columnName: 'boot_disk__id' })
  declare bootDiskId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Node, { localKey: 'id', foreignKey: 'nodeId' })
  declare node: BelongsTo<typeof Node>

  @belongsTo(() => BootDisk, { foreignKey: 'bootDiskId', localKey: 'id' })
  declare bootDisk: BelongsTo<typeof BootDisk>

  @belongsTo(() => Project, { localKey: 'id', foreignKey: 'projectId' })
  declare project: BelongsTo<typeof Project>
}
