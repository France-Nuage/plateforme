import { column, computed } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'

export default class AccountsController {
  public static table = 'billing.accounts'

  @computed()
  public get object() {
    return 'account'
  }

  @column({ isPrimary: true, columnName: 'account__id' })
  declare id: string

  @column.dateTime({ autoCreate: true })
  declare created_at: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updated_at: DateTime
}
