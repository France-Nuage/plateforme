import factory from '@adonisjs/lucid/factories'
import Cluster from '#models/infrastructure/cluster'

export const ClusterFactory = factory
  .define(Cluster, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      zoneId: faker.string.uuid(),
    }
  })
  .build()
