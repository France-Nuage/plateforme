import { BaseModel, column, manyToMany } from '@adonisjs/lucid/orm'
import type { PermissionId } from '@france-nuage/types'
import Role from '#models/iam/role'
import type { ManyToMany } from '@adonisjs/lucid/types/relations'
import { VerbId } from '#models/iam/verb'
import { ServiceId } from '#models/catalog/service'
import { TypeId } from '#models/iam/type'

export default class Permission extends BaseModel {
  public static table = 'iam.permissions'

  @column({ isPrimary: true, columnName: 'permission__id' })
  declare id: PermissionId

  @column()
  declare name: string

  @column({ isPrimary: true, columnName: 'service__id' })
  declare serviceId: ServiceId

  @column({ isPrimary: true, columnName: 'type__id' })
  declare typeId: TypeId

  @column({ isPrimary: true, columnName: 'verb__id' })
  declare verbId: VerbId

  @manyToMany(() => Role, {
    pivotTable: 'iam.role__permission',
    localKey: 'id',
    pivotForeignKey: 'permission__id',
    relatedKey: 'id',
    pivotRelatedForeignKey: 'role__id',
  })
  declare roles: ManyToMany<typeof Role>
}
