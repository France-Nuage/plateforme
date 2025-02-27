import factory from '@adonisjs/lucid/factories'
import { RegionFactory } from '#database/factories/infrastructure/region_factory'
import Country from '#models/localisation/country'

export const CountryFactory = factory
  .define(Country, ({ faker }) => {
    return {
      country__id: faker.string.uuid(),
      name: faker.address.country(),
      code: faker.address.countryCode(),
      latitude: faker.address.latitude(),
      longitude: faker.address.longitude(),
      postal_code_regex: faker.internet.password({
        length: 10,
        memorable: true,
        pattern: new RegExp(/\d/),
      }),
      phone_indicator: `+${faker.number.int({ min: 1, max: 999 })}`,
      phone_regex: faker.internet.password({
        length: 15,
        memorable: true,
        pattern: new RegExp(/[0-9]/),
      }),
      flag_svg: `<svg>${faker.lorem.paragraph()}</svg>`,
    }
  })
  .relation('regions', () => RegionFactory)
  .build()
