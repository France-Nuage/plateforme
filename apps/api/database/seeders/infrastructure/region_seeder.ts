import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Country from '#models/localisation/country'
import { RegionFactory } from '#database/factories/infrastructure/region_factory'

export default class ZoneSeeder extends BaseSeeder {
  public static async run() {
    const counties = await Country.all()
    for (const country of counties) {
      await RegionFactory.merge({ countryId: country.id }).createMany(2) // 2 zones par région
    }
  }
}
