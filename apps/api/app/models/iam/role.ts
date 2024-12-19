import { BaseModel, column, computed, manyToMany } from '@adonisjs/lucid/orm'
import type { ManyToMany } from '@adonisjs/lucid/types/relations'
import Permission from '#models/iam/permission'

export default class Role extends BaseModel {
  public static table = 'iam.roles'

  @computed()
  public get object() {
    return 'role'
  }

  @column({ isPrimary: true, columnName: 'role__id' })
  declare id: string

  @column({ columnName: 'service__id' })
  declare serviceId: string

  @column()
  declare description: string

  @manyToMany(() => Permission, {
    pivotTable: 'iam.role__permission',
    localKey: 'id',
    pivotForeignKey: 'role__id',
    relatedKey: 'id',
    pivotRelatedForeignKey: 'permission__id',
  })
  declare permissions: ManyToMany<typeof Permission>
}
