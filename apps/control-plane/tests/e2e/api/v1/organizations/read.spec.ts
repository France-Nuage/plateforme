import { organizations } from '#config/app'
import { randomUUID } from 'node:crypto'
import { expect, test, Users } from '../../../base.js'

test.describe('GET /api/v1/organizations/:id', () => {
  test('I can read a given organization', async ({ actingAs, organization, users }) => {
    // TODO: rely on a RBAC function rather than the admin user
    const { request } = await actingAs(users[Users.Admin])
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

  test('I cannot read a non-existing organization ', async ({ actingAs, organization, users }) => {
    // TODO: rely on a RBAC function rather than the admin user
    const { request } = await actingAs(users[Users.Admin])
    const response = await request.get(`/api/v1/organizations/${randomUUID()}`)
    const result = await response.json()

    expect(response.ok()).toBeFalsy()
    expect(response.status()).toBe(404)
  })
})
