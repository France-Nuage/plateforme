import { BaseModel, belongsTo, column, computed, hasMany } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import Organization from '#models/resource/organization'
import type { BelongsTo, HasMany } from '@adonisjs/lucid/types/relations'
import Project from '#models/resource/project'

export default class BillingAccount extends BaseModel {
  public static table = 'billing.accounts'

  @computed()
  public get object() {
    return 'billing_account'
  }

  @column({ isPrimary: true, columnName: 'account__id' })
  declare id: string

  @column()
  declare displayName: string

  @column()
  declare open: boolean

  @column()
  declare currencyCode: string

  @column({ columnName: 'organization__id' })
  declare organizationId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Organization, { foreignKey: 'organization__id', localKey: 'id' })
  declare organization: BelongsTo<typeof Organization>

  @hasMany(() => Project)
  declare project: HasMany<typeof Project>
}
