import { expect, test, Users } from '../../../base.js'

test.describe('POST /api/v1/auth/login', () => {
  test('I can authenticate with valid credentials', async ({ request, users }) => {
    const response = await request.post('/api/v1/auth/login', {
      data: {
        email: users[Users.Admin].email,
        password: 'password',
      },
    })
    const result = await response.json()

    expect(response.ok()).toBeTruthy()
    expect(response.status()).toBe(200)
    expect(Object.keys(result).sort()).toMatchObject([
      'abilities',
      'expiresAt',
      'lastUsedAt',
      'name',
      'token',
      'type',
    ])
    expect(result.type.toLowerCase()).toBe('bearer')
  })

  test('I cannot authenticate with invalid credentials', async ({ request, users }) => {
    const response = await request.post('/api/v1/auth/login', {
      data: {
        email: users[Users.Admin].email,
        password: 'an-invalid-password',
      },
    })
    const result = await response.json()

    expect(response.ok()).toBeFalsy()
    expect(response.status()).toBe(400)
    expect(result).toMatchObject({ errors: [{ message: 'Invalid user credentials' }] })
  })

  test('I am provided with validation messages on invalid content submission', async ({
    request,
  }) => {
    const response = await request.post('/api/v1/auth/login')
    const result = await response.json()

    expect(response.ok()).toBeFalsy()
    expect(response.status()).toBe(422)
    expect(result).toMatchObject({
      errors: [
        {
          message: 'The email field must be defined',
          rule: 'required',
          field: 'email',
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
