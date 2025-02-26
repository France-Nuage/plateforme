import { BaseModel, column, computed } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'

export default class InstanceType extends BaseModel {
  public static table = 'infrastructure.instance_types'

  @computed()
  public get object() {
    return 'instance_type'
  }

  @column({ isPrimary: true, columnName: 'instance_type__id' })
  declare id: string

  @column()
  declare name: string

  @column()
  declare description: string

  @column()
  declare vcpusMinGb: number

  @column()
  declare vcpusMaxGb: number

  @column()
  declare memoryMinGb: number

  @column()
  declare memoryMaxGb: number

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime
}
