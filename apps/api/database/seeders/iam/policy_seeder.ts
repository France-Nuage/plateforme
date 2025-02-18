import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Policy from '#models/iam/policy'
import config from '@adonisjs/core/services/config'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    const r = await Policy.updateOrCreateMany('id', [
      {
        id: config.get('app.rootOrganization.policy.id'),
        organizationId: config.get('app.rootOrganization.id'),
      },
    ])
    console.log('seeded pol', r)
  }
}
