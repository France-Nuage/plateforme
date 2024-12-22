import factory from '@adonisjs/lucid/factories'
import Cluster from '#models/infrastructure/cluster'
import { InstanceFactory } from '#database/factories/infrastructure/instance_factory'
import { NodeFactory } from '#database/factories/infrastructure/node_factory'

export const ClusterFactory = factory
  .define(Cluster, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      zoneId: faker.string.uuid(),
    }
  })
  .relation('nodes', () => NodeFactory)
  .build()
