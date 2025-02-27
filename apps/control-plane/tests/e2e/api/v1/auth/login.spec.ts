import { expect, test } from '@playwright/test'

test.describe('/api/v1/auth/login', () => {
  test('can authenticate with valid credentials', async ({ request }) => {
    const response = await request.post('/api/v1/auth/login', {
      data: {
        email: 'admin@france-nuage.fr',
        password: 'password',
      },
    })

    expect(response.ok()).toBeTruthy()
    expect(response.status()).toBe(200)
    const result = await response.json()

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
})
