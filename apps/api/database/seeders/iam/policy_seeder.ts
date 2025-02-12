import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Policy from '#models/iam/policy'
import config from '@adonisjs/core/services/config'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    await Policy.updateOrCreateMany('id', [
      {
        id: config.get('app.organizations.franceNuage.policyId'),
      },
    ])
  }
}
