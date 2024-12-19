import { BaseModel, belongsTo, column, computed } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'
import Service from '#models/service/service'

export default class Version extends BaseModel {
  public static table = 'version.versions'

  @computed()
  public get object() {
    return 'version'
  }

  @column({ isPrimary: true, columnName: 'version__id' })
  declare id: string

  @column()
  declare name: string

  @column()
  declare description: string

  @column.dateTime()
  declare availableAt: DateTime

  @column({ columnName: 'service__id' })
  declare serviceId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Service)
  declare service: BelongsTo<typeof Service>
}
