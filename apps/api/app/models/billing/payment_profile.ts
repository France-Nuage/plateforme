import {BaseModel, belongsTo, column, hasMany} from '@adonisjs/lucid/orm'
import {DateTime} from 'luxon'
import BillingAccount from '#models/billing/billing_account'
import Folder from '#models/resource/folder'
import type {BelongsTo, HasMany} from '@adonisjs/lucid/types/relations'

export default class PaymentProfile extends BaseModel {
  public static table = 'billing.payment_profile'

  @column({ isPrimary: true, columnName: 'payment_profile__id' })
  declare id: string

  @column()
  declare stripeCustomerId: string

  @column({ columnName: 'folder__id' })
  declare folderId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoUpdate: true, autoCreate: true })
  declare updatedAt: DateTime

  @hasMany(() => BillingAccount)
  declare billingAccounts: HasMany<typeof BillingAccount>

  @belongsTo(() => Folder)
  declare folder: BelongsTo<typeof Folder>
}
