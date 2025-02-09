import { BaseSeeder } from '@adonisjs/lucid/seeders'
import { ServiceId } from '#models/catalog/service'
import Role, { RoleId } from '#models/iam/role'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    await Role.updateOrCreateMany('id', [
      {
        id: RoleId.OrganizationAdmin,
        serviceId: ServiceId.ResourceManager,
      },
      {
        id: RoleId.ProjectAdmin,
        serviceId: ServiceId.ResourceManager,
      },
    ])
  }
}
