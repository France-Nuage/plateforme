import { User } from '@france-nuage/types'
import { expect, request, test as base } from '@playwright/test'
import type { PlaywrightTestConfig } from '@playwright/test'
import config from '../../playwright.config.js'

/**
 * Re-exports unmodified Playwright tools as-in
 */
export { expect }

/**
 * The available fixtured users.
 */
enum FixtureUser {
  Admin,
}

/**
 * The authentication data for a user.
 */
type Credentials = {
  token: string
}

/**
 * The fixtures exposed in the tests.
 *
 * **Usage**
 *
 * ```js
 * import { expect, test } from '#/base';
 * test('example test', async ({ request, users }) => {
 *    const response = await request.get('/api/v1/auth/me', { headers: {
 *      'Authorization': `Bearer ${users[FixtureUser.Admin].token}`
 *    }})
 *    expect(response.status()).toBe(200)
 * })
 * ````
 */
type Fixtures = {
  users: Record<FixtureUser, Pick<User, 'email'> & Credentials>
}

export const test = base.extend<{}, Fixtures>({
  users: [
    async ({}, use) => {
      const admin = { email: 'admin@france-nuage.fr' }
      const credentials = await login(admin, config)

      await use({
        [FixtureUser.Admin]: { ...admin, ...credentials },
      })
    },
    { scope: 'worker' },
  ],
})

async function login(
  user: Pick<User, 'email'>,
  config: PlaywrightTestConfig
): Promise<Credentials> {
  const context = await request.newContext(config.use)

  const response = await context.post('/api/v1/auth/login', {
    data: {
      email: 'admin@france-nuage.fr',
      password: 'password',
    },
  })
  const credentials = await response.json()

  return credentials.token
}
