import { BaseModel, column, hasMany } from '@adonisjs/lucid/orm'
import type { HasMany } from '@adonisjs/lucid/types/relations'
import Type from '#models/iam/type'

export default class Service extends BaseModel {
  public static table = 'catalog.services'

  @column({ isPrimary: true, columnName: 'service__id' })
  declare id: string

  @column()
  declare description: string

  @hasMany(() => Type)
  declare types: HasMany<typeof Type>
}

export enum ServiceId {
  CloudAssets = 'cloudassets',
  Compute = 'compute',
  ResourceManager = 'resourcemanager',
}
