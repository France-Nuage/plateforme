import axios from 'axios'
import Zone from '#models/infrastructure/zone'
import RequestQueryBuilder from '#utils/request_query_builder'
import Instance from '#models/infrastructure/instance'
import Price from '#models/billing/price'
import { proxmoxApi } from '#utils/proxmox_helper'

const getNextVMID = async (url: string, token: string) => {
  try {
    const response = await axios.get(`${url}/api2/json/cluster/nextid`, {
      headers: {
        Authorization: token,
      },
    })
    return response.data.data
  } catch (e) {
    throw new Error('Could not get next VMID')
    // throw new Error(e)
  }
}

export default {
  list: async function ({
    includes,
    page,
    perPage,
  }: {
    includes?: Array<string>
    page?: number
    perPage?: number
  }) {
    return new RequestQueryBuilder(Instance.query())
      .withIncludes(includes)
      .withPagination(page, perPage)
      .apply()
  },
  get: async function (id: string, includes: Array<string>) {
    return new RequestQueryBuilder(Instance.query())
      .withIncludes(includes)
      .applyWhere([['id', '=', id]])
      .firstOrFail()
  },
  create: async function ({ zoneId, name }: { zoneId: string; name: string }) {
    const zone = await new RequestQueryBuilder(Zone.query())
      .withIncludes(['clusters.nodes'])
      .applyWhere([['id', zoneId]])
      .firstOrFail()

    if (!zone.clusters[0]?.nodes[0]) {
      return new Error('No nodes available in the zone')
    }

    const node = zone.clusters[0].nodes[0]

    const vmid = await getNextVMID(node.url, node.token)
    await proxmoxApi.node.qemu.create(
      {
        vmid,
        nodeName: node.name,
        token: node.token,
        url: node.url,
      },
      {
        name,
      }
    )
    return await Instance.create({
      name: name,
      pveVmId: vmid,
      nodeId: node.id,
    })
  },
  update: async function (instance: Instance, data: Partial<Instance>) {
    return instance.merge(data).save()
  },
  getCurrentPrice: async (options: { zoneId: string; cpu: number; ram: number }) => {
    const pricingList = await Price.query()
      .where('zone__id', options.zoneId)
      .andWhereIn('resource_type', ['CPU', 'RAM'])

    const currentPriceRam = pricingList.find((price) => price.resourceType === 'RAM')
    const currentPriceCpu = pricingList.find((price) => price.resourceType === 'CPU')

    if (!currentPriceRam || !currentPriceCpu) {
      return 0
    }

    const ramPricing = currentPriceRam.pricePerUnit * options.ram
    const cpuPricing = currentPriceCpu.pricePerUnit * options.cpu

    return {
      cpu: cpuPricing,
      ram: ramPricing,
      totalHourlyPrice: { amount: ramPricing + cpuPricing },
      totalMounthlyPrice: { amount: (ramPricing + cpuPricing) * 24 * 30 },
    }
  },
}
