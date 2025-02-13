import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Country, { CountryCode } from '#models/localisation/country'
import Region, { RegionName } from '#models/infrastructure/region'
import Zone, { ZoneName } from '#models/infrastructure/zone'
import Cluster from '#models/infrastructure/cluster'
import config from '@adonisjs/core/services/config'

export default class extends BaseSeeder {
  static environment = ['development', 'testing']

  public async run() {
    const france = await Country.findByOrFail({ code: CountryCode.France })
    const vendee = await Region.findByOrFail({ name: RegionName.Vendee, countryId: france.id })
    const vendeeA = await Zone.findByOrFail({ name: ZoneName.FranceVendeeA, regionId: vendee.id })

    await Cluster.updateOrCreateMany('id', [
      {
        id: config.get('dev.cluster.id'),
        name: config.get('dev.cluster.name'),
        zoneId: vendeeA.id,
      },
    ])
  }
}
