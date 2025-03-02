import { expect, test, Users } from '../../../base.js'

test.describe('GET /api/v1/organizations/:id', () => {
  test('I can read a given organization', async ({ actingAs, users }) => {
    // TODO: rely on a RBAC function rather than the admin user
    const { request } = await actingAs(users[Users.Admin])
    // TODO: pull organization id from fixture
    const response = await request.get('/api/v1/organizations/00000000-0000-0000-0000-000000000000')
    const result = await response.json()

    expect(response.ok()).toBeTruthy()
    expect(response.status()).toBe(200)
    expect(result).toEqual('okiedokie')
  })
})
