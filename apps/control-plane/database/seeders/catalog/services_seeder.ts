import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Service, { ServiceId } from '#models/catalog/service'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    await Service.updateOrCreateMany(
      'id',
      Object.values(ServiceId).map((service) => ({
        id: service,
      }))
    )
  }
}
