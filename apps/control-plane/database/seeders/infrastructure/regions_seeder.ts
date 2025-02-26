import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Country, { CountryCode } from '#models/localisation/country'
import Region, { RegionName } from '#models/infrastructure/region'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    const france = await Country.findByOrFail({ code: CountryCode.France })
    await Region.updateOrCreateMany('name', [
      {
        name: RegionName.LoireAtlantique,
        countryId: france.id,
      },
      {
        name: RegionName.Vendee,
        countryId: france.id,
      },
    ])
  }
}
