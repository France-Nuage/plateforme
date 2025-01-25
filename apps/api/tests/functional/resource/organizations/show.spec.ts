// import { test } from '@japa/runner'
// import testUtils from '@adonisjs/core/services/test_utils'
//
// const defaultUserPayload = {
//   lastname: 'Doe',
//   firstname: 'Jhon',
//   email: 'jhon.doe@france-nuage.com',
//   password: '1234567890',
// }

// test.group('organization_show', (group) => {
//   group.each.setup(() => testUtils.db().withGlobalTransaction())
//   group.each.setup(() => testUtils.db().truncate())
//
//   test('show an organizations', async ({ client, assert }) => {
//     const responseUser = await client.post('/api/v1/auth/register').json(defaultUserPayload)
//     const token = responseUser.body().token.token
//
//     const response = await client.get('/api/v1/organizations').headers({
//       authorization: `Bearer ${token}`,
//     })
//     const body = response.body()
//
//     response.assertStatus(200)
//     assert.equal(body.data.length, 1)
//     assert.equal(body.data[0].name, 'Doe-organization')
//     assert.equal(body.data[0].object, 'organization')
//   })
// })
