import { BaseSeeder } from '@adonisjs/lucid/seeders'
import { ZoneFactory } from '#database/factories/infrastructure/zone_factory'
import Region from '#models/infrastructure/region'

export default class ZoneSeeder extends BaseSeeder {
  public static async run() {
    const regions = await Region.all()
    for (const region of regions) {
      await ZoneFactory.merge({ regionId: region.id }).createMany(2) // 2 zones par région
    }
  }
}
