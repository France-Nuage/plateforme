import { randomUUID } from 'node:crypto'
import { expect, test } from '../../../base.js'
import { PermissionId } from '@france-nuage/types'

test.describe('GET /api/v1/organizations/:id', () => {
  test('I can read a given organization', async ({ actingWith: actingAs, organization }) => {
    const { request } = await actingAs(PermissionId.ResourceManagerOrganizationsGet, organization)
    const response = await request.get(`/api/v1/organizations/${organization.id}`)
    const result = await response.json()

    expect(response.ok()).toBeTruthy()
    expect(response.status()).toBe(200)
    expect(Object.keys(result).sort()).toEqual(['createdAt', 'id', 'name', 'object', 'updatedAt'])
  })

  test('I cannot read a given organization without being authenticated', async ({
    organization,
    request,
  }) => {
    const response = await request.get(`/api/v1/organizations/${organization.id}`)
    const result = await response.json()

    expect(response.ok()).toBeFalsy()
    expect(response.status()).toBe(401)
    expect(result).toEqual({ errors: [{ message: 'Unauthorized access' }] })
  })

  test('I cannot read a non-existing organization ', async ({
    actingWith: actingAs,
    organization,
  }) => {
    const { request } = await actingAs(PermissionId.ResourceManagerOrganizationsGet, organization)
    const response = await request.get(`/api/v1/organizations/${randomUUID()}`)

    expect(response.ok()).toBeFalsy()
    expect(response.status()).toBe(404)
  })
})
