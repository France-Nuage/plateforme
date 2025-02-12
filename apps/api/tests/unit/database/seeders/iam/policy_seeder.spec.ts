import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import Policy from '#models/iam/policy'
import config from '@adonisjs/core/services/config'

test.group('Policy Seeder', (group) => {
  group.each.setup(async () => {
    await testUtils.db().withGlobalTransaction()
    await testUtils.db().truncate()
    await testUtils.db().seed()
  })

  test('it should seed the database with a policy for the franceNuage organization', async ({
    assert,
  }) => {
    // Assert the database has the expected entries
    assert.exists(await Policy.find(config.get('app.organizations.franceNuage.policyId')))
  })
})
