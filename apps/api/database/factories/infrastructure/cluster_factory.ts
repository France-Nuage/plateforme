import factory from '@adonisjs/lucid/factories'
import Cluster from '#models/infrastructure/cluster'
import { NodeFactory } from '#database/factories/infrastructure/node_factory'

export const ClusterFactory = factory
  .define(Cluster, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      host: faker.internet.domainName(),
      name: faker.string.uuid(),
      tokenId: 'user@dev-cluster',
      tokenSecret: faker.string.uuid(),
      zoneId: faker.string.uuid(),
    }
  })
  .relation('nodes', () => NodeFactory)
  .build()
