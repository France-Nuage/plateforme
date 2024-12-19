import { BaseSeeder } from '@adonisjs/lucid/seeders'
import { ServiceFactory } from '#database/factories/service/service_factory'
import { VersionFactory } from '#database/factories/service/version_factory'

export default class extends BaseSeeder {
  public static async run() {
    const services = await ServiceFactory.createMany(5) // Crée 5 services

    for (const service of services) {
      await VersionFactory.merge({ serviceId: service.id }).createMany(3) // 3 versions par service
    }
  }
}
