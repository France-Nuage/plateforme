import { BaseModel, column, computed, hasMany } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { HasMany } from '@adonisjs/lucid/types/relations'
import Folder from '#models/resource/folder'
import BillingAccount from '#models/billing/billing_account'

export default class Organization extends BaseModel {
  public static table = 'resource.organizations'

  @computed()
  public get object() {
    return 'organization'
  }

  @column({ isPrimary: true, columnName: 'organization__id' })
  declare id: string

  @column()
  declare name: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @hasMany(() => Folder)
  declare folders: HasMany<typeof Folder>

  @hasMany(() => BillingAccount)
  declare billingAccounts: HasMany<typeof BillingAccount>
}
