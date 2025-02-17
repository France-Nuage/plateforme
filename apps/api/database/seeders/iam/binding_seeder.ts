import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Binding from '#models/iam/binding'
import { RoleId } from '#models/iam/role'
import { ServiceId } from '#models/catalog/service'
import User from '#models/user'
import config from '@adonisjs/core/services/config'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    const workerUser = await User.findByOrFail('email', config.get('app.worker.email'))

    await Binding.updateOrCreateMany(
      ['policyId', 'memberId'],
      [
        {
          policyId: config.get('app.rootOrganization.policy.id'),
          memberId: workerUser.id,
          roleId: RoleId.Worker,
          serviceId: ServiceId.ResourceManager,
        },
      ]
    )

    // Skip the next steps in production environment
    if (config.get('app.environment') === 'production') {
      return
    }

    const devUser = await User.findByOrFail('email', config.get('dev.user.email'))

    await Binding.updateOrCreateMany(
      ['policyId', 'memberId'],
      [
        {
          policyId: config.get('app.rootOrganization.policy.id'),
          memberId: devUser.id,
          roleId: RoleId.OrganizationAdmin,
          serviceId: ServiceId.ResourceManager,
        },
      ]
    )
  }
}
