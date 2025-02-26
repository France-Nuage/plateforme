import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'

test.group('forgot-password', (group) => {
  group.each.setup(() => testUtils.db().withGlobalTransaction())
  group.each.setup(() => testUtils.db().truncate())
})
