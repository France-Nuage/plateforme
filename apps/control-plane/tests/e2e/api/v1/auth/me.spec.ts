import { expect, test, Users } from '../../../base.js'

test.describe('GET /api/v1/auth/me', () => {
  test('I can retrieve my info as a user', async ({ actingAs, users }) => {
    const admin = users[Users.Admin]
    const { request } = await actingAs(admin)
    const response = await request.get('/api/v1/auth/me')
    const result = await response.json()

    expect(response.status()).toBe(200)
    expect(Object.keys(result).sort()).toEqual([
      'createdAt',
      'email',
      'firstname',
      'id',
      'lastname',
      'object',
      'updatedAt',
    ])
    expect(result).toMatchObject({
      email: admin.email,
      firstname: admin.firstname,
      lastname: admin.lastname,
      object: 'user',
    })
  })

  test('I cannot retrieve my info without being authenticated', async ({ request }) => {
    const response = await request.get('/api/v1/auth/me')
    const result = await response.json()

    expect(response.status()).toBe(401)
    expect(result).toEqual({ errors: [{ message: 'Unauthorized access' }] })
  })
})
