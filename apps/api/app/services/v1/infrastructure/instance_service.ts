import Zone from '#models/infrastructure/zone'
import RequestQueryBuilder from '#utils/request_query_builder'
import Instance from '#models/infrastructure/instance'
import Price from '#models/billing/price'

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
    const zone: Zone = await new RequestQueryBuilder(Zone.query())
      .withIncludes(['clusters.nodes'])
      .applyWhere([['id', zoneId]])
      .firstOrFail()

    if (!zone.clusters[0]?.nodes[0]) {
      return new Error('No nodes available in the zone')
    }

    const cluster = zone.clusters[0]
    const node = cluster.nodes[0]

    const hypervisor = cluster.api()

    const id = await hypervisor.getNextInstanceId()
    await hypervisor.node(node).instance(id).create({ name })

    return await Instance.create({
      name: name,
      pveVmId: id,
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
