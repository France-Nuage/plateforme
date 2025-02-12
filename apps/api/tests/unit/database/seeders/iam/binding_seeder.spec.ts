import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import User from '#models/user'
import Binding from '#models/iam/binding'
import config from '@adonisjs/core/services/config'

test.group('Binding Seeder', (group) => {
  group.each.setup(async () => {
    await testUtils.db().withGlobalTransaction()
    await testUtils.db().truncate()
    await testUtils.db().seed()
  })

  test('it should seed the database with a binding for associating the worker user to the franceNuage organization', async ({
    assert,
  }) => {
    const workerUser = await User.findByOrFail('email', config.get('app.worker.email'))

    // Assert the database has the expected entries
    assert.exists(
      await Binding.findByOrFail({
        policyId: config.get('app.organizations.franceNuage.policyId'),
        memberId: workerUser.id,
      })
    )
  })
})
