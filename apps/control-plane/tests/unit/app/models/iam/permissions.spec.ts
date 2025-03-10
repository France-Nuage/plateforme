import { test } from '@japa/runner'
import Permission from '#models/iam/permission'
import { PermissionId } from '@france-nuage/types'

test.group('Permission', () => {
  test('every variant of the PermissionId enum exists in the database', async ({ assert }) => {
    const permissions = await Permission.query().select('permission__id')
    const permissionIds = permissions.map((permission) => permission.id)

    for (const permissionId of Object.values(PermissionId)) {
      assert.include(
        permissionIds,
        permissionId,
        `Permission ${permissionId} missing in the database`
      )
    }
  })

  test('every permission in the database exists in the PermissionId enum', async ({ assert }) => {
    const permissions = await Permission.query().select('permission__id')
    const permissionIds = permissions.map((permission) => permission.id)

    for (const permissionId of permissionIds) {
      assert.include(
        Object.values(PermissionId),
        permissionId,
        `Permission ${permissionId} missing in the PermissionId enum`
      )
    }
  })
})
