import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import config from '@adonisjs/core/services/config'
import User from '#models/user'

test.group('Users Seeder', (group) => {
  group.each.setup(async () => {
    await testUtils.db().withGlobalTransaction()
    await testUtils.db().truncate()
    await testUtils.db().seed()
  })

  test('it should seed the database with a user for the worker', async ({ assert }) => {
    // Assert the database has the expected entries
    assert.exists(await User.findByOrFail({ email: config.get('app.worker.email') }))
  })
})
