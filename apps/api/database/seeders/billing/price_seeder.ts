import {BaseSeeder} from '@adonisjs/lucid/seeders'
import {PriceFactory} from '#database/factories/billing/price_factory'

export default class extends BaseSeeder {
  public async run() {
    await PriceFactory.merge([
      {
        resourceType: 'CPU',
        resourceUnit: 'vCPU',
        // pricingUnit: 'GIBIBYTE_HOUR',
        zoneId: '00000000-0000-0000-0000-000000000003',
        pricePerUnit: 0.5,
      },
      {
        resourceType: 'RAM',
        resourceUnit: 'Gb',
        pricingUnit: 'GIBIBYTE_HOUR',
        zoneId: '00000000-0000-0000-0000-000000000003',
        pricePerUnit: 1.5,
      },
    ]).createMany(2)
  }
}
