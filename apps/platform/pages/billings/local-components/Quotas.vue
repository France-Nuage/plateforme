<template>
  <div class="grid grid-cols-12">
    <div class="col-span-3">
      <div>
        <h1 class="mb-2 text-xl">Usage sommaire</h1>
        <p class="text-sm text-gray-400">
          Your plan includes a limited amount of usage.
          If exceeded, you may experience restrictions, as you are currently not billed for overages.
          It may take up to 1 hour to refresh.
        </p>
      </div>
      <div class="mt-8">
        <h2 class="mb-2 text-sm">More informations</h2>
        <c-sidebar-navigation-item title="Guides" icon="square-bottom-up-line-duotone" to="/instances" />
        <c-sidebar-navigation-item title="Api référence" icon="square-bottom-up-line-duotone" to="/services" />
      </div>
    </div>
    <div class="col-span-9">

      <div class="grid grid-cols-3">

        <c-metric-card class="col-span-12 md:col-span-1" title="Request number" :total-value="1000" :partial-value="100" unit="Req" />
        <c-metric-card class="col-span-12 md:col-span-1" title="Request number" :total-value="1000" :partial-value="200" unit="GB" />
        <c-metric-card class="col-span-12 md:col-span-1" title="Request number" :total-value="1000" :partial-value="300" unit="GB" />

        <c-metric-card class="col-span-12 md:col-span-1" title="Request number" :total-value="1000" :partial-value="400" unit="GB" />
        <c-metric-card class="col-span-12 md:col-span-1" title="Request number" :total-value="1000" :partial-value="500" unit="GB" />
        <c-metric-card class="col-span-12 md:col-span-1" title="Request number" :total-value="1000" :partial-value="600" unit="GB" />

        <c-metric-card class="col-span-12 md:col-span-1" title="Request number" :total-value="1000" :partial-value="900" unit="GB" />
        <c-metric-card class="col-span-12 md:col-span-1" title="Request number" :total-value="1000" :partial-value="800" unit="GB" />
        <c-metric-card class="col-span-12 md:col-span-1" title="Request number" :total-value="1000" :partial-value="100" unit="GB" />

      </div>
    </div>
  </div>

  <div class="mt-16">
    <c-table
      name="quotas"
      :data="quotas"
      :headers="headers"
    >
      <template #col-percent="{ entity, key }">
        <c-linear-gauge :percent="entity[key]" />
      </template>
    </c-table>
  </div>
</template>

<script setup lang="ts">
import CSidebarNavigationItem from "~/components/sidebar/CSidebarNavigationItem.vue";
import CMetricCard from "~/components/metric/CMetricCard.vue";
import CTable from "~/components/table/CTable.vue";
import CLinearGauge from "~/components/gauge/CLinearGauge.vue";

const headers = [
  { key: "service.name", label: "Service" },
  { key: "name", label: "Nom" },
  { key: "type.name", label: "Type" },
  { key: "totalValue", label: "Value" },
  { key: "percent", label: "Pourcentage d'utilisation actuel" },
  { key: "currentUsage", label: "Utilisation actuelle" },
]

const quotas = ref([
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 10, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 20, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 30, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 40, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 50, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 60, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 70, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 80, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 90, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 100, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 0, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 8.6, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 20, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 8.6, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 20, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 8.6, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 20, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 8.6, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 20, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 8.6, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 20, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 8.6, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Networks', currentUsage: 1, totalValue: 5, percent: 20, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
  { name: 'Subnetwork ranges per VPC Network', currentUsage: 43, totalValue: 500, percent: 8.6, service: { name: 'Compute Engine API' }, type: { name: 'Quota' } },
])
</script>
