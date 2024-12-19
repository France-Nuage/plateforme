import factory from '@adonisjs/lucid/factories'
import Region from '#models/infrastructure/region'
import {ZoneFactory} from "#database/factories/infrastructure/zone_factory";

export const RegionFactory = factory
  .define(Region, ({ faker }) => {
    return {
      region__id: faker.string.uuid(),
      name: faker.address.state(),
      country__id: faker.string.uuid(),
      createdAt: faker.date.recent(),
      updatedAt: faker.date.future(),
    }
  })
  .relation('zones', () => ZoneFactory)
  .build()
