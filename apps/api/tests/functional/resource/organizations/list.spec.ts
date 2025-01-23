import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import organization_service from '#services/v1/resource/organization_service'
import User from '#models/user'

const defaultUserPayload = {
  lastname: 'Doe',
  firstname: 'Jhon',
  email: 'jhon.doe@france-nuage.com',
  password: '1234567890',
}

test.group('organization_list', (group) => {
  group.each.setup(() => testUtils.db().withGlobalTransaction())
  group.each.setup(() => testUtils.db().truncate())
  group.each.setup(async () => {
    const user1 = await User.create({
      email: defaultUserPayload.email,
      password: defaultUserPayload.password,
    })
    await organization_service.create({ name: 'Doe-organization' }, user1)

    // const user2 = await User.create({
    //   email: defaultUserPayload.email,
    //   password: defaultUserPayload.password,
    // })
    // await organization_service.create({ name: 'not john doe organization' }, user2)
  })

  test('list all organizations', async ({ client, assert }) => {
    const responseUser = await client.post('/api/v1/auth/token').json(defaultUserPayload)
    const token = responseUser.body().token.token

    const response = await client.get('/api/v1/organizations').headers({
      authorization: `Bearer ${token}`,
    })
    const body = response.body()

    response.assertStatus(200)
    assert.equal(body.data.length, 1)
    assert.equal(body.data[0].name, 'Doe-organization')
    assert.equal(body.data[0].object, 'organization')
  })
})
