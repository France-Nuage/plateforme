import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'

const defaultUserPayload = {
  lastname: 'Doe',
  firstname: 'Jhon',
  email: 'jhon.doe@france-nuage.com',
  password: '1234567890',
}

test.group('auth_register', (group) => {
  group.each.setup(() => testUtils.db().withGlobalTransaction())
  group.each.setup(() => testUtils.db().truncate())

  test('register user', async ({ client, assert }) => {
    const response = await client.post('/api/v1/auth/register').json(defaultUserPayload)
    const body = response.body()
    response.assertStatus(201)

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

  test('register user failed 422 error with short password', async ({ client, assert }) => {
    const response = await client
      .post('/api/v1/auth/register')
      .json({ ...defaultUserPayload, password: '12345' })

    response.assertStatus(422)
    assert.snapshot(response.body()).match()
  })

  test('register user failed 422 error with missed information', async ({ client, assert }) => {
    const response = await client
      .post('/api/v1/auth/register')
      .json({ email: defaultUserPayload.email, password: '1234567890' })

    response.assertStatus(422)
    assert.snapshot(response.body()).match()
  })

  test('register user failed 422 error with bad format user', async ({ client, assert }) => {
    const response = await client
      .post('/api/v1/auth/register')
      .json({ email: defaultUserPayload.email, password: '1234567890' })

    response.assertStatus(422)
    assert.snapshot(response.body()).match()
  })

  test('register user fail 422 error with email already exist', async ({ client, assert }) => {
    await client.post('/api/v1/auth/register').json(defaultUserPayload)

    const response = await client.post('/api/v1/auth/register').json(defaultUserPayload)

    response.assertStatus(422)

    assert.snapshot(response.body()).match()
  })
})
