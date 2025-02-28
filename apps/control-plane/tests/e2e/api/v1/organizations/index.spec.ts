import { expect, test, Users } from '../../../base.js'

test.describe('GET /api/v1/organizations', () => {
  test('I can retrieve a list of organizations', async ({ actingAs, users }) => {
    // TODO: rely on a RBAC function rather than the admin user
    const { request } = await actingAs(users[Users.Admin])
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
    expect(Object.keys(result.data[0]).sort()).toEqual([
      'createdAt',
      'id',
      'name',
      'object',
      'updatedAt',
    ])
  })

  test('The items have the expected shape', async ({ actingAs, users }) => {
    // TODO: rely on a RBAC function rather than the admin user
    // TODO: once a second list endpoint is tested, generalize the shape test
    const { request } = await actingAs(users[Users.Admin])
    const response = await request.get('/api/v1/organizations')
    const result = await response.json()

    expect(response.ok()).toBeTruthy()
    expect(response.status()).toBe(200)
    expect(result.data.length).toBeGreaterThan(0)
    expect(Object.keys(result.data[0]).sort()).toEqual([
      'createdAt',
      'id',
      'name',
      'object',
      'updatedAt',
    ])
  })

  test('The endpoint is paginated', async ({ actingAs, users }) => {
    // TODO: rely on a RBAC function rather than the admin user
    // TODO: once a second list endpoint is tested, generalize the pagination test
    const { request } = await actingAs(users[Users.Admin])
    const response = await request.get('/api/v1/organizations')
    const result = await response.json()

    expect(response.ok()).toBeTruthy()
    expect(response.status()).toBe(200)
    expect(Object.keys(result.meta).sort()).toMatchObject([
      'currentPage',
      'firstPage',
      'firstPageUrl',
      'lastPage',
      'lastPageUrl',
      'nextPageUrl',
      'perPage',
      'previousPageUrl',
      'total',
    ])
    expect(result.meta.currentPage).toBe(1)
    expect(result.meta.firstPage).toBe(1)
    expect(result.meta.lastPage).toBeGreaterThan(0)
    expect(result.meta.perPage).toBe(10)
    expect(result.meta.total).toBeGreaterThan(0)
  })

  test('I cannot retrieve a list of organizations while not authenticated', async ({ request }) => {
    const response = await request.get('/api/v1/organizations')
    const result = await response.json()

    expect(response.ok()).toBeFalsy()
    expect(response.status()).toBe(401)
    expect(result).toEqual({ errors: [{ message: 'Unauthorized access' }] })
  })
})
