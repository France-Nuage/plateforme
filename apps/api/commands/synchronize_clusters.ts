import { BaseCommand } from '@adonisjs/core/ace'
import type { CommandOptions } from '@adonisjs/core/types/ace'
import Cluster from '#models/infrastructure/cluster'
import { synchronizeCluster } from '#services/synchronization'

export default class extends BaseCommand {
  static commandName = 'app:clusters:synchronize'
  static description = ''

  static options: CommandOptions = { startApp: true }

  /**
   * Synchronize every cluster registered in the database
   */
  async run() {
    const clusters = await Cluster.all()

    const work = clusters.map(synchronizeCluster)
    await Promise.all(work)
  }
}
