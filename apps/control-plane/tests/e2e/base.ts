import { ofetch } from 'ofetch'
import { organizationsRepository } from '@france-nuage/api'
import { Organization, User } from '@france-nuage/types'
import { expect, request as baseRequest, test as base } from '@playwright/test'
import type { APIRequestContext, PlaywrightTestConfig } from '@playwright/test'
import baseConfig from '../../playwright.config.js'

/**
 * Re-exports unmodified Playwright tools as-in
 */
export { expect }

/**
 * The available fixtured users.
 */
export enum Users {
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
  /**
   * Acts as the given user.
   *
   * This function returns a promise containing a contextualized request object
   * for performing authenticated actions on behalf of the given user.
   *
   * **Usage**
   *
   * ```js
   * test('example test', async ({ actingAs }) => {
   *  const { request } = actingAs(FixtureUser.Admin);
   *  const response = await request.post('/api/v1/destroy-all-evidences');
   *  expect(response.status).toBe(200)
   * })
   * ```
   */
  actingAs: (user: Credentials) => Promise<{ request: APIRequestContext }>

  /**
   * The pre-defined organization.
   */
  organization: Organization

  /**
   * The pre-defined users.
   */
  users: Record<Users, Pick<User, 'email' | 'firstname' | 'lastname'> & Credentials>
}

export const test = base.extend<{}, Fixtures>({
  organization: [
    async ({}, use) => {
      const organization = await organizationsRepository(ofetch, {}).read(
        '00000000-0000-0000-0000-000000000000'
      )
      console.log('organization should have been retrieved')
      use(organization)
    },
    { scope: 'worker' },
  ],
  /**
   * Provides an implementation for the `users` fixtures.
   *
   * The users defined in this fixture are generated altogether on the first
   * fixture import. This means running a single test will generate all the
   * fixtures. In order to keep tests fast and scoped, avoid adding extra users
   * to this dictionary and prefer relying on the `actingAs` fixture function to
   * generate scoped, lazy users authenticated against the control-plane and
   * matching the tested RBAC requirements.
   *
   * TODO: Once the RBAC tests are stabilized, move out of a dictionary pattern
   * and expose those users as single fixtures. I forecast the need for a single
   * "admin" user, alongside some top-level resources (e.g. an organization).
   */
  users: [
    async ({}, use) => {
      const admin = { email: 'admin@france-nuage.fr', firstname: 'Wile E.', lastname: 'Coyote' }
      const credentials = await login(admin, baseConfig)

      await use({
        [Users.Admin]: { ...admin, ...credentials },
      })
    },
    { scope: 'worker' },
  ],

  /**
   * Provides an implementation for the `actingAs` fixture function.
   *
   * This implementation creates a new context for the given user, relying on
   * the `users` fixture to access an existing (and created in database) user.
   *
   * I tried to provide a dynamic approach, instantiating a user and creating it
   * (in the database) lazyily, however this requires tweaking Playwright
   * behavior and overridding his types and behavior too much. Relying on a list
   * of pre-existing users (that are still dynamically created for every ).
   *
   * An alternative approach would be to define a stateful collection keeping
   * track of created users, and change the function signature to accept a
   * Role => Permission association. This would allow to lazily create a user,
   * authenticate it, and then reuse it across the worker. This behavior should
   * be implemented alongside the tests for the RBAC model, instead of bloating
   * the `users` fixtures. Note that we still need the users fixtures to access
   * a privileged admin user that is able to create the required users matching
   * the tested RBAC scope.
   *
   * TODO: implement RBAC management for this function, then update the function
   * documentation
   */
  actingAs: [
    async ({}, use) => {
      await use(async (user: Credentials) => {
        // Create a new context request with authentication headers
        const request = await baseRequest.newContext({
          ...baseConfig.use,
          extraHTTPHeaders: {
            ...baseConfig.use?.extraHTTPHeaders,
            Authorization: `Bearer ${user.token}`,
          },
        })

        // Return the contextualized request object
        return { request }
      })
    },
    { scope: 'worker' },
  ],
})

/**
 * Logs the given user in.
 */
async function login(
  user: Pick<User, 'email'>,
  config: PlaywrightTestConfig
): Promise<Credentials> {
  // Create a new context -- this avoids mistakenly using an authenticated one.
  const context = await baseRequest.newContext(config.use)

  // Submit a login request to generate credentials.
  const response = await context.post('/api/v1/auth/login', {
    data: {
      email: user.email,
      password: 'password',
    },
  })
  const credentials = await response.json()

  // Return the generated credentials.
  return { token: credentials.token }
}
