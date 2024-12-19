import { BaseModel, belongsTo, column, computed, hasMany } from '@adonisjs/lucid/orm'
import { DateTime } from 'luxon'
import type { BelongsTo, HasMany } from '@adonisjs/lucid/types/relations'
import Project from '#models/resource/project'

export default class Folder extends BaseModel {
  public static table = 'resource.folders'

  @computed()
  public get object() {
    return 'folder'
  }

  @column({ isPrimary: true, columnName: 'folder__id' })
  declare id: string

  @column()
  declare name: string

  @column({ columnName: 'organization__id' })
  declare organizationId: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @belongsTo(() => Folder, { localKey: 'id', foreignKey: 'organizationId' })
  declare organization: BelongsTo<typeof Folder>

  @hasMany(() => Project)
  declare projects: HasMany<typeof Project>
}
