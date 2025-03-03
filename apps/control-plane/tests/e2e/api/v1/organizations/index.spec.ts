import { PermissionId } from '@france-nuage/types'
import { expect, test } from '../../../base.js'

test.describe('GET /api/v1/organizations', () => {
  test('I can retrieve a list of organizations', async ({ actingWith }) => {
    const { request } = await actingWith(PermissionId.ResourceManagerOrganizationsList)
    const response = await request.get('/api/v1/organizations')
    const result = await response.json()

    expect(response.ok()).toBeTruthy()
    expect(response.status()).toBe(200)
    expect(result).toMatchObject({
      data: [
        {
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

  test('The items have the expected shape', async ({ actingWith }) => {
    const { request } = await actingWith(PermissionId.ResourceManagerOrganizationsList)
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

  test('The endpoint is paginated', async ({ actingWith }) => {
    const { request } = await actingWith(PermissionId.ResourceManagerOrganizationsList)
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
