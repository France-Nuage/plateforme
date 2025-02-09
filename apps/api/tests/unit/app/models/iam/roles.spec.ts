import { test } from '@japa/runner'
import Role, { RoleId } from '#models/iam/role'

test.group('Role', () => {
  test('every variant of the RoleId enum exists in the database', async ({ assert }) => {
    const roles = await Role.query().select('role__id')
    const roleIds = roles.map((role) => role.id)

    for (const roleId of Object.values(RoleId)) {
      assert.include(roleIds, roleId, `Role ${roleId} missing in the database`)
    }
  })

  test('every role in the database exists in the RoleId enum', async ({ assert }) => {
    const roles = await Role.query().select('role__id')
    const roleIds = roles.map((role) => role.id)

    for (const roleId of roleIds) {
      assert.include(Object.values(RoleId), roleId, `Verb ${roleId} missing in the RoleId enum`)
    }
  })
})
