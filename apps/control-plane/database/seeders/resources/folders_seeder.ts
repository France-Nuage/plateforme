import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Organization from '#models/resource/organization'
import config from '@adonisjs/core/services/config'
import Folder from '#models/resource/folder'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    const organization = await Organization.findOrFail(config.get('app.rootOrganization.id'))

    await Folder.updateOrCreateMany(
      ['name', 'organizationId'],
      [
        {
          organizationId: organization.id,
          name: config.get('app.defaultFolder.name'),
        },
      ]
    )
  }
}
