import { BaseModel, belongsTo, column } from '@adonisjs/lucid/orm'
import Service from '#models/catalog/service'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'

export default class Type extends BaseModel {
  public static table = 'iam.types'

  @column({ isPrimary: true, columnName: 'type__id' })
  declare id: string

  @column({ isPrimary: true, columnName: 'service__id' })
  declare serviceId: string

  @column()
  declare description: string

  @belongsTo(() => Service)
  declare service: BelongsTo<typeof Service>
}
