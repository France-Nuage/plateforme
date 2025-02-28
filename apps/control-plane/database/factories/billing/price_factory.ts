import factory from '@adonisjs/lucid/factories'
import Price from '#models/billing/price'

export const PriceFactory = factory
  .define(Price, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      pricePerUnit: faker.number.int({ min: 10, max: 100 }),
      resourceType: 'RAM',
      resourceUnit: 'Gb',
      pricingUnit: 'GIBIBYTE_HOUR ',
      zoneId: faker.string.uuid(),
    }
  })
  .build()
