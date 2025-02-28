import { expect, test } from '../../../base.js'

test.describe('GET /api/v1/organizations', () => {
  test('I can retrieve a list of organizations', async ({ request }) => {
    const response = await request.get('/api/v1/organizations')
    const result = await response.json()

    expect(response.ok()).toBeTruthy()
    expect(response.status()).toBe(200)

    expect(result).toMatchObject({
      data: [
        {
          id: '00000000-0000-0000-0000-000000000000',
          name: 'France Nuage',
          object: 'organization',
        },
      ],
    })
  })

  test('I am provided with pagination', async ({ request }) => {})

  test('I cannot retrieve a list of organizations while not authenticated', async ({ request }) => {
    const response = await request.get('/api/v1/organizations')
    const result = await response.json()

    expect(response.ok()).toBeFalsy()
    expect(response.status()).toBe(401)
    console.log(result)

    expect(result).toEqual({ errors: [{ message: 'Unauthorized access' }] })
  })
})
