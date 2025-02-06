import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'

test.group('migrations', (group) => {
  group.each.setup(() => testUtils.db().withGlobalTransaction())

  test('ensure migrations can be run', async () => {
    // run the migrations and then rollback
    await testUtils.db().migrate()
  })

  test('ensure migrations can be rolled back', async () => {
    // run the migrations and then rollback
    const rollback = await testUtils.db().migrate()
    await rollback()
  })
})
