import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import User from '#models/user'

const defaultUserPayload = {
  lastname: 'Doe',
  firstname: 'Jhon',
  email: 'jhon.doe@france-nuage.com',
  password: '1234567890',
}

let user: any = null

test.group('auth_login', (group) => {
  group.each.setup(() => testUtils.db().withGlobalTransaction())
  group.each.setup(async () => {
    user = await User.create(defaultUserPayload)
  })
  group.each.setup(() => testUtils.db().truncate())

  test('login user', async ({ client, assert }) => {
    const response = await client.post('/api/v1/auth/token').json({
      email: user.email,
      password: '1234567890',
    })

    const body = response.body()
    response.assertStatus(200)

    assert.equal(body.token.type, 'bearer')
    assert.equal(body.token.name, null)
    assert.equal(body.token.token.slice(0, 4), 'oat_')
    assert
      .snapshot({
        lastname: body['lastname'],
        firstname: body['firstname'],
        email: body['email'],
        object: body['object'],
      })
      .match()
  })
  test('login user fail 422 error', async ({ client, assert }) => {
    const response = await client.post('/api/v1/auth/token').json({})

    response.assertStatus(422)
    assert.snapshot(response.body()).match()
  })
})
