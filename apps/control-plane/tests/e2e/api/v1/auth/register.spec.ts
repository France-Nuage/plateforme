import { faker } from '@faker-js/faker'
import { expect, test } from '../../../base.js'

test.describe('POST /api/v1/auth/register', () => {
  test('I can register with valid data without being authenticated', async ({ request }) => {
    const response = await request.post('/api/v1/auth/register', {
      data: {
        email: faker.internet.email(),
        password: faker.internet.password(),
        firstname: faker.person.firstName(),
        lastname: faker.person.lastName(),
      },
    })
    const result = await response.json()

    expect(response.ok()).toBeTruthy()
    expect(response.status()).toBe(201)
    expect(Object.keys(result.token).sort()).toEqual([
      'abilities',
      'expiresAt',
      'lastUsedAt',
      'name',
      'token',
      'type',
    ])
    expect(Object.keys(result.user).sort()).toEqual([
      'createdAt',
      'email',
      'firstname',
      'id',
      'lastname',
      'object',
      'updatedAt',
    ])
  })

  test('I can register with valid data while being authenticated', async ({ actingWith }) => {
    const { request } = await actingWith()
    const response = await request.post('/api/v1/auth/register', {
      data: {
        email: faker.internet.email(),
        password: faker.internet.password(),
        firstname: faker.person.firstName(),
        lastname: faker.person.lastName(),
      },
    })

    expect(response.ok()).toBeTruthy()
    expect(response.status()).toBe(201)
  })

  test('I am provided with validation messages on invalid content submission', async ({
    request,
  }) => {
    const response = await request.post('/api/v1/auth/register')
    const result = await response.json()

    expect(response.ok()).toBeFalsy()
    expect(response.status()).toBe(422)
    expect(result).toEqual({
      errors: [
        {
          message: 'The email field must be defined',
          rule: 'required',
          field: 'email',
        },
        {
          message: 'The lastname field must be defined',
          rule: 'required',
          field: 'lastname',
        },
        {
          message: 'The firstname field must be defined',
          rule: 'required',
          field: 'firstname',
        },
        {
          message: 'The password field must be defined',
          rule: 'required',
          field: 'password',
        },
      ],
    })
  })
})
