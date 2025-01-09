import Price from '#models/billing/price'

export default {
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
      total: ramPricing + cpuPricing,
    }
  },
  getPriceForPeriod: async (zoneId: string, period: { start: string, end: string }) => {
    const prices = await Price.query()
      .where('zone', zoneId)
      // .where('resource_type', resourceType)
      .where('effective_start', '<', period.end)
      .andWhere((query) => {
        query.where('effective_end', '>', period.start).orWhereNull('effective_end')
      })
      .orderBy('effective_start', 'asc')

    if (prices.length === 0) {
      // Si aucun tarif trouvé (cas théorique), on ignore ou on loggue une erreur
    }
  },
}
