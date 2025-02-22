import { BaseModel, column, manyToMany } from '@adonisjs/lucid/orm'
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

export enum PermissionId {
  CloudAssetsAssetsListIamRoles = 'cloudassets.assets.listIamRoles',
  ComputeImagesGet = 'compute.images.get',
  ComputeImagesList = 'compute.images.list',
  ComputeInstancesCreate = 'compute.instances.create',
  ComputeInstancesGet = 'compute.instances.get',
  ComputeInstancesList = 'compute.instances.list',
  ComputeInstancesUpdate = 'compute.instances.update',
  ComputeRegionsGet = 'compute.regions.get',
  ComputeRegionsList = 'compute.regions.list',
  ComputeZonesGet = 'compute.zones.get',
  ComputeZonesList = 'compute.zones.list',
  ResourceManagerFoldersCreate = 'resourcemanager.folders.create',
  ResourceManagerFoldersGet = 'resourcemanager.folders.get',
  ResourceManagerFoldersGetIamPolicy = 'resourcemanager.folders.getIamPolicy',
  ResourceManagerFoldersList = 'resourcemanager.folders.list',
  ResourceManagerFoldersSetIamPolicy = 'resourcemanager.folders.setIamPolicy',
  ResourceManagerOrganizationsGet = 'resourcemanager.organizations.get',
  ResourceManagerOrganizationsGetIamPolicy = 'resourcemanager.organizations.getIamPolicy',
  ResourceManagerOrganizationsList = 'resourcemanager.organizations.list',
  ResourceManagerOrganizationsSetIamPolicy = 'resourcemanager.organizations.setIamPolicy',
  ResourceManagerProjectsCreate = 'resourcemanager.projects.create',
  ResourceManagerProjectsGet = 'resourcemanager.projects.get',
  ResourceManagerProjectsGetIamPolicy = 'resourcemanager.projects.getIamPolicy',
  ResourceManagerProjectsList = 'resourcemanager.projects.list',
  ResourceManagerProjectsSetIamPolicy = 'resourcemanager.projects.setIamPolicy',
}
