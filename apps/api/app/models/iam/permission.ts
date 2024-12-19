import { BaseModel, column, manyToMany } from '@adonisjs/lucid/orm'
import Role from '#models/iam/role'
import type { ManyToMany } from '@adonisjs/lucid/types/relations'

export default class Permission extends BaseModel {
  public static table = 'iam.permissions'

  @column({ isPrimary: true, columnName: 'permission__id' })
  declare id: string

  @column()
  declare name: string

  @column({ isPrimary: true, columnName: 'service__id' })
  declare serviceId: string

  @column({ isPrimary: true, columnName: 'type__id' })
  declare typeId: string

  @column({ isPrimary: true, columnName: 'verb__id' })
  declare verbId: string

  @manyToMany(() => Role,{
    pivotTable: 'iam.role__permission',
    localKey: 'id',
    pivotForeignKey: 'permission__id',
    relatedKey: 'id',
    pivotRelatedForeignKey: 'role__id',
  })
  declare roles: ManyToMany<typeof Role>
}
