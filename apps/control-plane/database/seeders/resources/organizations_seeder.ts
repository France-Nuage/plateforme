import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Organization from '#models/resource/organization'
import config from '@adonisjs/core/services/config'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    await Organization.updateOrCreateMany('id', [
      {
        id: config.get('app.rootOrganization.id'),
        name: config.get('app.rootOrganization.name'),
      },
    ])
  }
}
