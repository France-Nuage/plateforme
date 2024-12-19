import factory from '@adonisjs/lucid/factories'
import Zone from '#models/infrastructure/zone'
import { InstanceFactory } from '#database/factories/infrastructure/instance_factory'

export const ZoneFactory = factory
  .define(Zone, ({ faker }) => {
    return {
      zone__id: faker.string.uuid(),
      name: faker.address.city(),
      region__id: faker.string.uuid(),
      createdAt: faker.date.recent(),
      updatedAt: faker.date.future(),
    }
  })
  .relation('instances', () => InstanceFactory)
  .build()
