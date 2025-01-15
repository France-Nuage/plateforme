import {BaseModel, column, computed, hasMany} from '@adonisjs/lucid/orm'
import {DateTime} from 'luxon'
import OsVersion from '#models/infrastructure/os_version'
import type {HasMany} from '@adonisjs/lucid/types/relations'

export default class BootDisk extends BaseModel {
  public static table = 'infrastructure.boot_disks'

  @computed()
  public get object() {
    return 'boot_disk'
  }

  @column({ isPrimary: true, columnName: 'boot_disk__id' })
  declare id: string

  @column()
  declare os: string

  @column()
  declare disk_type: string

  @column()
  declare size_gb: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime

  @hasMany(() => OsVersion)
  declare osVersion: HasMany<typeof OsVersion>
}
