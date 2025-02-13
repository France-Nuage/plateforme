import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Country, { CountryCode } from '#models/localisation/country'
import Region, { RegionName } from '#models/infrastructure/region'
import Zone, { ZoneName } from '#models/infrastructure/zone'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    const france = await Country.findByOrFail({ code: CountryCode.France })
    const vendee = await Region.findByOrFail({ name: RegionName.Vendee, countryId: france.id })

    await Zone.updateOrCreateMany('name', [
      {
        name: ZoneName.FranceVendeeA,
        regionId: vendee.id,
      },
    ])
  }
}
