import {BaseModel, belongsTo, column} from '@adonisjs/lucid/orm'
import type {BelongsTo} from '@adonisjs/lucid/types/relations'
import Role from '#models/iam/role'
import User from '#models/user'
import Service from '#models/catalog/service'
import Policy from '#models/iam/policy'

export default class Binding extends BaseModel {
  public static table = 'iam.user_resource_policy_binding'

  @column({ columnName: 'binding__id', isPrimary: true })
  declare id: string

  @column({ columnName: 'member__id' })
  declare memberId: number

  @column({ columnName: 'service__id' })
  declare serviceId: string

  @column({ columnName: 'role__id' })
  declare roleId: string

  @column({ columnName: 'policy__id' })
  declare policyId: string

  @belongsTo(() => Role, { localKey: 'id', foreignKey: 'roleId' })
  declare role: BelongsTo<typeof Role>

  @belongsTo(() => User, { localKey: 'id', foreignKey: 'memberId' })
  declare user: BelongsTo<typeof User>

  @belongsTo(() => Service, { localKey: 'id', foreignKey: 'serviceId' })
  declare service: BelongsTo<typeof Service>

  @belongsTo(() => Policy, { localKey: 'id', foreignKey: 'policyId' })
  declare policy: BelongsTo<typeof Policy>
}
