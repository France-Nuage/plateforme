import { DateTime } from 'luxon'
import hash from '@adonisjs/core/services/hash'
import { compose } from '@adonisjs/core/helpers'
import { BaseModel, column, computed, hasMany, manyToMany } from '@adonisjs/lucid/orm'
import { withAuthFinder } from '@adonisjs/auth/mixins/lucid'
import { DbAccessTokensProvider } from '@adonisjs/auth/access_tokens'
import type { HasMany, ManyToMany } from '@adonisjs/lucid/types/relations'
import Binding from '#models/iam/binding'
import Policy from '#models/iam/policy'

const AuthFinder = withAuthFinder(() => hash.use('scrypt'), {
  uids: ['email'],
  passwordColumnName: 'password',
})

export default class User extends compose(BaseModel, AuthFinder) {
  public static table = 'member.users'

  @computed()
  public get object() {
    return 'user'
  }

  @column({ isPrimary: true })
  declare id: number

  @column()
  declare lastname: string | null

  @column()
  declare firstname: string | null

  @column()
  declare email: string

  @column({ serializeAs: null })
  declare password: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime | null

  @hasMany(() => Binding, { foreignKey: 'memberId', localKey: 'id' })
  declare bindings: HasMany<typeof Binding>

  @manyToMany(() => Policy, {
    pivotTable: 'iam.user_resource_policy_binding',
    localKey: 'id',
    pivotForeignKey: 'member__id',
    relatedKey: 'id',
    pivotRelatedForeignKey: 'policy__id',
  })
  declare policies: ManyToMany<typeof Policy>

  static accessTokens = DbAccessTokensProvider.forModel(User, {
    expiresIn: '30 days',
    prefix: 'oat_',
    table: 'iam.auth_access_tokens',
    type: 'auth_token',
    tokenSecretLength: 40,
  })
}
