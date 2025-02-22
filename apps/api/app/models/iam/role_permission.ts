import { BaseModel, belongsTo, column } from '@adonisjs/lucid/orm'
import Role from '#models/iam/role'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'
import Permission from '#models/iam/permission'

export default class RolePermission extends BaseModel {
  public static table = 'iam.role__permission'
  public static selfAssignPrimaryKey = true

  @column({ isPrimary: true, columnName: 'role__id' })
  declare roleId: string

  @column({ isPrimary: true, columnName: 'permission__id' })
  declare permissionId: string

  @belongsTo(() => Role, { localKey: 'roleId', foreignKey: 'roleId' })
  declare role: BelongsTo<typeof Role>

  @belongsTo(() => Permission, { localKey: 'roleId', foreignKey: 'roleId' })
  declare permission: BelongsTo<typeof Permission>
}
