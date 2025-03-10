import { BaseSeeder } from '@adonisjs/lucid/seeders'
import { PermissionId } from '@france-nuage/types'
import { RoleId } from '#models/iam/role'
import RolePermission from '#models/iam/role_permission'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    await RolePermission.updateOrCreateMany(
      ['roleId', 'permissionId'],
      [
        {
          roleId: RoleId.OrganizationAdmin,
          permissionId: PermissionId.ResourceManagerFoldersCreate,
        },
        {
          roleId: RoleId.OrganizationAdmin,
          permissionId: PermissionId.ResourceManagerFoldersGet,
        },
        {
          roleId: RoleId.OrganizationAdmin,
          permissionId: PermissionId.ResourceManagerFoldersGetIamPolicy,
        },
        {
          roleId: RoleId.OrganizationAdmin,
          permissionId: PermissionId.ResourceManagerFoldersList,
        },
        {
          roleId: RoleId.OrganizationAdmin,
          permissionId: PermissionId.ResourceManagerOrganizationsGet,
        },
        {
          roleId: RoleId.OrganizationAdmin,
          permissionId: PermissionId.ResourceManagerOrganizationsGetIamPolicy,
        },
        {
          roleId: RoleId.OrganizationAdmin,
          permissionId: PermissionId.ResourceManagerOrganizationsList,
        },
        {
          roleId: RoleId.OrganizationAdmin,
          permissionId: PermissionId.ResourceManagerProjectsGet,
        },
        {
          roleId: RoleId.OrganizationAdmin,
          permissionId: PermissionId.ResourceManagerProjectsGetIamPolicy,
        },
        {
          roleId: RoleId.ProjectAdmin,
          permissionId: PermissionId.ResourceManagerFoldersCreate,
        },
        {
          roleId: RoleId.ProjectAdmin,
          permissionId: PermissionId.ResourceManagerFoldersGet,
        },
        {
          roleId: RoleId.ProjectAdmin,
          permissionId: PermissionId.ResourceManagerFoldersGetIamPolicy,
        },
        {
          roleId: RoleId.ProjectAdmin,
          permissionId: PermissionId.ResourceManagerFoldersList,
        },
        {
          roleId: RoleId.ProjectAdmin,
          permissionId: PermissionId.ResourceManagerProjectsGet,
        },
        {
          roleId: RoleId.ProjectAdmin,
          permissionId: PermissionId.ResourceManagerProjectsGetIamPolicy,
        },
        {
          roleId: RoleId.ProjectAdmin,
          permissionId: PermissionId.ResourceManagerProjectsList,
        },
        {
          roleId: RoleId.Worker,
          permissionId: PermissionId.ComputeInstancesList,
        },
        {
          roleId: RoleId.Worker,
          permissionId: PermissionId.ComputeInstancesUpdate,
        },
      ]
    )
  }
}
