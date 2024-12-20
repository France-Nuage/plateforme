import factory from '@adonisjs/lucid/factories'
import Region from '#models/infrastructure/region'
import { ZoneFactory } from '#database/factories/infrastructure/zone_factory'

export const RegionFactory = factory
  .define(Region, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      name: faker.lorem.sentence(),
      countryId: faker.string.uuid(),
    }
  })
  .relation('zones', () => ZoneFactory)
  .build()
