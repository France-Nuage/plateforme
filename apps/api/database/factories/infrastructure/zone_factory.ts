import factory from '@adonisjs/lucid/factories'
import Zone from '#models/infrastructure/zone'
import { InstanceFactory } from '#database/factories/infrastructure/instance_factory'

export const ZoneFactory = factory
  .define(Zone, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      regionId: faker.string.uuid(),
    }
  })
  .relation('instances', () => InstanceFactory)
  .build()
