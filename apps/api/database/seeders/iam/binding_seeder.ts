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
          policyId: config.get('app.organizations.franceNuage.policyId'),
          memberId: workerUser.id,
          roleId: RoleId.Worker,
          serviceId: ServiceId.ResourceManager,
        },
      ]
    )
  }
}
