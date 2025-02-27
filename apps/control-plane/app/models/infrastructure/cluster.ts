import { BaseModel, belongsTo, column, hasMany } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo, HasMany } from '@adonisjs/lucid/types/relations'
import Zone from '#models/infrastructure/zone'
import Node from '#models/infrastructure/node'
import { getProxmoxClusterHypervisorApi } from '#services/hypervisor/proxmox.api'
import { HypervisorApi } from '#services/hypervisor/hypervisor.api'
import Organization from '#models/resource/organization'

export default class Cluster extends BaseModel {
  public static table = 'infrastructure.clusters'

  @column({ isPrimary: true, columnName: 'cluster__id' })
  declare id: string

  @column({ columnName: 'organization__id' })
  declare organizationId: string

  @column({ columnName: 'zone__id' })
  declare zoneId: string

  @column()
  declare name: string

  @column()
  declare host: string

  @column({ columnName: 'token_id' })
  declare tokenId: string

  @column({ columnName: 'token_secret' })
  declare tokenSecret: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @hasMany(() => Node)
  declare nodes: HasMany<typeof Node>

  @belongsTo(() => Zone, { localKey: 'id', foreignKey: 'zoneId' })
  declare zone: BelongsTo<typeof Zone>

  @belongsTo(() => Organization, { localKey: 'id', foreignKey: 'organizationId' })
  declare organization: BelongsTo<typeof Organization>

  /**
   * Get an instance of the distant hypervisor cluster API.
   */
  api(): HypervisorApi {
    return getProxmoxClusterHypervisorApi(this)
  }
}
