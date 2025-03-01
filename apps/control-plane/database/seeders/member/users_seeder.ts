import { BaseSeeder } from '@adonisjs/lucid/seeders'
import User from '#models/user'
import config from '@adonisjs/core/services/config'
import db from '@adonisjs/lucid/services/db'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    await User.updateOrCreate(
      {
        email: config.get('app.worker.email'),
      },
      {
        password: '', // we don't want a password as the worker user should not perform a login action
      }
    )

    // Skip the next steps in production environment
    if (config.get('app.environment') === 'production') {
      return
    }

    // Create the demo users
    await User.updateOrCreateMany('email', [
      {
        email: config.get('dev.user.email'),
        password: config.get('dev.user.password'),
        firstname: 'Wile E.',
        lastname: 'Coyote',
      },
    ])

    // Generate an entry for the worker access token
    await this.generateWorkerAccessToken()
  }

  /**
   * Generate an entry in database matching the worker access token when the conditions are met.
   */
  private async generateWorkerAccessToken() {
    // The next statement checks if the auth_access_tokens table is empty; which should be the case when seeding the
    // database. When it happens, the following statement inserts a row that maps to the WORKER_API_TOKEN env var, so
    // the worker can operate in an authenticated state by default when booting the project with docker-compose.
    const result = await db.rawQuery(`SELECT MAX(id) AS max_id FROM iam.auth_access_tokens`)
    const isAuthAccessTokenTableEmpty = result.rows[0].max_id === null

    // The token hashing algorithm takes the id as an input, which requires the inserted token to be a predictable id,
    // namely "1". We thus check the auth_access_tokens serial id is uninitialized to ensure the inserted `id` will be
    // `1`.
    if (isAuthAccessTokenTableEmpty) {
      await db.table('iam.auth_access_tokens').insert({
        id: 1,
        tokenable_id: 1,
        type: 'auth_token',
        name: null,
        hash: '246b8c5a7327f0de6f46b20a2f8cefe0dd14b1f2f48c38f62afcee136aae0a4a',
        abilities: '["*"]',
        created_at: '2025-02-10T15:08:51.005Z',
        updated_at: '2025-02-10T15:08:51.005Z,',
        last_used_at: null,
        expires_at: '2035-02-11T03:08:51.004Z',
      })

      // Reset the id serial to 1, which avoids conflicts on id
      await db.rawQuery(`SELECT setval('iam.auth_access_tokens_id_seq', 1, true)`)
    }
  }
}
