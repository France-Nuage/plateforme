import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import { RoleId } from '#models/iam/role'
import RolePermission from '#models/iam/role_permission'
import { PermissionId } from '#models/iam/permission'

test.group('RolePermission Seeder', (group) => {
  group.each.setup(async () => {
    await testUtils.db().withGlobalTransaction()
    await testUtils.db().truncate()
    await testUtils.db().seed()
  })

  test('it should seed the database with permission associations for the worker role', async ({
    assert,
  }) => {
    // Assert the worker role has the `compute.instances.list` permission
    assert.exists(
      await RolePermission.findByOrFail({
        roleId: RoleId.Worker,
        permissionId: PermissionId.ComputeInstancesList,
      })
    )
    // Assert the worker role has the `compute.instances.update` permission
    assert.exists(
      await RolePermission.findByOrFail({
        roleId: RoleId.Worker,
        permissionId: PermissionId.ComputeInstancesUpdate,
      })
    )
  })
})
