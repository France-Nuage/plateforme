import { test } from '@japa/runner'
import { RoleId } from '#models/iam/role'
import RolePermission from '#models/iam/role_permission'

test.group('RolePermission', () => {
  test('every Role has some associated permissions in the database', async ({ assert }) => {
    const bindings = await RolePermission.query()
    const roleIds = bindings.map((binding) => binding.roleId)

    for (const roleId of Object.values(RoleId)) {
      assert.include(
        roleIds,
        roleId,
        `Role ${roleId} does not have any associated permissions in the database`
      )
    }
  })
})
