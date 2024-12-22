import { BaseSeeder } from '@adonisjs/lucid/seeders'
import { CountryFactory } from '#database/factories/localisation/country_factory'
import { RegionFactory } from '#database/factories/infrastructure/region_factory'
import { ZoneFactory } from '#database/factories/infrastructure/zone_factory'
import { ClusterFactory } from '#database/factories/infrastructure/cluster_factory'
import { NodeFactory } from '#database/factories/infrastructure/node_factory'

export default class extends BaseSeeder {
  public async run() {
    await CountryFactory.merge([
      {
        id: '00000000-0000-0000-0000-0000000000f1',
        code: 'FR',
        name: 'France',
        phoneIndicator: '+33',
        phoneRegex: '^\\+33\\s?[1-9](\\s?\\d{2}){4}$',
        postalCodeRegex: '^\\d{5}$',
        latitude: 46.603354,
        longitude: 1.888334,
        flagSvg:
          '<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="#fff" d="M10 4H22V28H10z"></path><path d="M5,4h6V28H5c-2.208,0-4-1.792-4-4V8c0-2.208,1.792-4,4-4Z" fill="#092050"></path><path d="M25,4h6V28h-6c-2.208,0-4-1.792-4-4V8c0-2.208,1.792-4,4-4Z" transform="rotate(180 26 16)" fill="#be2a2c"></path><path d="M27,4H5c-2.209,0-4,1.791-4,4V24c0,2.209,1.791,4,4,4H27c2.209,0,4-1.791,4-4V8c0-2.209-1.791-4-4-4Zm3,20c0,1.654-1.346,3-3,3H5c-1.654,0-3-1.346-3-3V8c0-1.654,1.346-3,3-3H27c1.654,0,3,1.346,3,3V24Z" opacity=".15"></path><path d="M27,5H5c-1.657,0-3,1.343-3,3v1c0-1.657,1.343-3,3-3H27c1.657,0,3,1.343,3,3v-1c0-1.657-1.343-3-3-3Z" fill="#fff" opacity=".2"></path></svg>',
      },
    ]).createMany(1)

    await RegionFactory.merge([
      {
        id: '00000000-0000-0000-0000-000000000001',
        name: 'Vendée',
        countryId: '00000000-0000-0000-0000-0000000000f1',
      },
      {
        id: '00000000-0000-0000-0000-000000000002',
        name: 'Loire Atlantique',
        countryId: '00000000-0000-0000-0000-0000000000f1',
      },
    ]).createMany(2)

    await ZoneFactory.merge([
      {
        id: '00000000-0000-0000-0000-000000000003',
        name: 'Vendée a',
        regionId: '00000000-0000-0000-0000-000000000001',
      },
      {
        id: '00000000-0000-0000-0000-000000000004',
        name: 'Vendée b',
        regionId: '00000000-0000-0000-0000-000000000001',
      },
    ]).createMany(2)

    await ClusterFactory.merge([
      {
        id: '00000000-0000-0000-0000-000000000005',
        zoneId: '00000000-0000-0000-0000-000000000003',
      },
    ]).createMany(1)

    await NodeFactory.merge([
      {
        id: '00000000-0000-0000-0000-000000000006',
        url: 'https://proxmox-poc-node-1.france-nuage.fr',
        name: 'pve-node1',
        token: 'PVEAPIToken=root@pam!api=0a253801-d1c0-4e74-964e-da6b61ffe92c',
        clusterId: '00000000-0000-0000-0000-000000000005',
      },
      {
        id: '00000000-0000-0000-0000-000000000007',
        url: 'https://proxmox-poc-node-2.france-nuage.fr',
        name: 'pve-node2',
        token: 'PVEAPIToken=root@pam!api=0a253801-d1c0-4e74-964e-da6b61ffe92c',
        clusterId: '00000000-0000-0000-0000-000000000005',
      },
      {
        id: '00000000-0000-0000-0000-000000000008',
        url: 'https://proxmox-poc-node-3.france-nuage.fr',
        name: 'pve-node3',
        token: 'PVEAPIToken=root@pam!api=0a253801-d1c0-4e74-964e-da6b61ffe92c',
        clusterId: '00000000-0000-0000-0000-000000000005',
      },
    ]).createMany(3)
  }
}
