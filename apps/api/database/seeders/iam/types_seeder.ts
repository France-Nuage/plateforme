import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Type, { TypeId } from '#models/iam/type'
import { ServiceId } from '#models/catalog/service'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    await Type.updateOrCreateMany('id', [
      {
        id: TypeId.Assets,
        serviceId: ServiceId.CloudAssets,
      },
      {
        id: TypeId.Folders,
        serviceId: ServiceId.ResourceManager,
      },
      {
        id: TypeId.Images,
        serviceId: ServiceId.Compute,
      },
      {
        id: TypeId.Instances,
        serviceId: ServiceId.Compute,
      },
      {
        id: TypeId.Organizations,
        serviceId: ServiceId.ResourceManager,
      },
      {
        id: TypeId.Projects,
        serviceId: ServiceId.ResourceManager,
      },
      {
        id: TypeId.Regions,
        serviceId: ServiceId.Compute,
      },
      {
        id: TypeId.Zones,
        serviceId: ServiceId.Compute,
      },
    ])
  }
}
