import { BaseSeeder } from '@adonisjs/lucid/seeders'
import config from '@adonisjs/core/services/config'
import Node from '#models/infrastructure/node'

export default class extends BaseSeeder {
  static environment = ['development', 'testing']

  public async run() {
    await Node.updateOrCreateMany('id', [
      {
        id: config.get('dev.node.id'),
        clusterId: config.get('dev.cluster.id'),
        name: config.get('dev.node.name'),
        url: config.get('dev.node.url'),
        token: config.get('dev.node.token'),
      },
    ])
  }
}
