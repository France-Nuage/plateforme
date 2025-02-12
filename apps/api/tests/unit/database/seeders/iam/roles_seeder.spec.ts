import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import Role, { RoleId } from '#models/iam/role'

test.group('Roles Seeder', (group) => {
  group.each.setup(async () => {
    await testUtils.db().withGlobalTransaction()
    await testUtils.db().truncate()
    await testUtils.db().seed()
  })

  test('it should seed the database with a role for the worker', async ({ assert }) => {
    // Assert the database has the expected entries
    assert.exists(await Role.find(RoleId.Worker))
  })
})
