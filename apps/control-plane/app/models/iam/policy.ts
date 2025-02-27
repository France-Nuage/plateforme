import { BaseModel, belongsTo, column, manyToMany } from '@adonisjs/lucid/orm'
import type { BelongsTo, ManyToMany } from '@adonisjs/lucid/types/relations'
import Organization from '#models/resource/organization'
import Project from '#models/resource/project'
import Folder from '#models/resource/folder'
import User from '#models/user'

export default class Policy extends BaseModel {
  public static table = 'iam.resource_policy'

  @column({ columnName: 'policy__id' })
  declare id: string

  @column({ columnName: 'organization__id' })
  declare organizationId: string

  @column({ columnName: 'folder__id' })
  declare folderId: string

  @column({ columnName: 'project__id' })
  declare projectId: string

  @belongsTo(() => Organization)
  declare organization: BelongsTo<typeof Organization>

  @belongsTo(() => Project)
  declare project: BelongsTo<typeof Project>

  @belongsTo(() => Folder)
  declare folder: BelongsTo<typeof Folder>

  @manyToMany(() => User, {
    pivotTable: 'iam.user_resource_policy_binding',
    localKey: 'id',
    pivotForeignKey: 'policy__id',
    relatedKey: 'id',
    pivotRelatedForeignKey: 'member__id',
  })
  declare users: ManyToMany<typeof User>
}
