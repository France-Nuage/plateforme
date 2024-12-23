import factory from '@adonisjs/lucid/factories'
import Zone from '#models/infrastructure/zone'
import { ClusterFactory } from '#database/factories/infrastructure/cluster_factory'

export const ZoneFactory = factory
  .define(Zone, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      name: faker.lorem.sentence(),
      regionId: faker.string.uuid(),
    }
  })
  .relation('clusters', () => ClusterFactory)
  .build()
