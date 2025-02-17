import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Organization from '#models/resource/organization'
import config from '@adonisjs/core/services/config'
import Project from '#models/resource/project'
import Folder from '#models/resource/folder'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    const organization = await Organization.findOrFail(config.get('app.rootOrganization.id'))

    const folder = await Folder.findByOrFail({
      organizationId: organization.id,
      name: 'Interne',
    })

    await Project.updateOrCreateMany(
      ['folderId', 'name'],
      [
        {
          folderId: folder.id,
          name: config.get('app.defaultProject.name'),
        },
      ]
    )
  }
}
