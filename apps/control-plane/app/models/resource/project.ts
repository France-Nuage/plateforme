import { BaseModel, belongsTo, column, computed } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'
import Folder from '#models/resource/folder'
import BillingAccount from '#models/billing/billing_account'

export default class Project extends BaseModel {
  public static table = 'resource.projects'

  @computed()
  public get object() {
    return 'project'
  }

  @column({ isPrimary: true, columnName: 'project__id' })
  declare id: string

  @column()
  declare name: string

  @column({ columnName: 'billing_account__id' })
  declare billingAccountId: string

  @column({ columnName: 'folder__id' })
  declare folderId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Folder, {
    localKey: 'id',
    foreignKey: 'folderId',
  })
  declare folder: BelongsTo<typeof Folder>

  @belongsTo(() => BillingAccount, { localKey: 'id', foreignKey: 'billingAccountId' })
  declare billingAccount: BelongsTo<typeof BillingAccount>
}
