import { test } from '@japa/runner'
import testUtils from "@adonisjs/core/services/test_utils";

test.group('Folders store', (group) => {
  group.each.setup(() => testUtils.db().withGlobalTransaction())
  group.each.setup(() => testUtils.db().truncate())

  test('store a folder', async ({ client }) => {
    const response = await client.post('/folders')

    response.assertStatus(200)
    response.assertBody({
      id: 1,
      email: 'foo@bar.com',
    })
  })
})
