import { BaseSeeder } from '@adonisjs/lucid/seeders'
import { ClusterFactory } from '#database/factories/infrastructure/cluster_factory'
import Zone from '#models/infrastructure/zone'

export default class ClusterSeeder extends BaseSeeder {
  public static async run() {
    const zones = await Zone.all()
    for (const zone of zones) {
      await ClusterFactory.merge({ zoneId: zone.id }).createMany(3) // 3 clusters par zone
    }
  }
}
