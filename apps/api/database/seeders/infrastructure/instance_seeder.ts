import { BaseSeeder } from '@adonisjs/lucid/seeders'
import { InstanceFactory } from '#database/factories/infrastructure/instance_factory'
import Cluster from '#models/infrastructure/cluster'

export default class extends BaseSeeder {
  public static async run() {
    const clusters = await Cluster.all()
    for (const cluster of clusters) {
      await InstanceFactory.merge({ clusterId: cluster.id }).createMany(5) // 5 instances par cluster
    }
  }
}
