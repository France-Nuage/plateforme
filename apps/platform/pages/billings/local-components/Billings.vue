<template>
  <div class="grid grid-cols-12 gap-20">

    <div class="col-span-8 flex flex-col gap-16">

      <div class="grid grid-cols-2 gap-4">
        <div class="px-6 py-8 rounded-lg border-2 dark:bg-gray-900 dark:border-gray-800">
          <span class="block text-sm dark:text-gray-400">Previous Month</span>
          <span class="block text-2xl font-semibold">3 143,86 $US</span>
        </div>
        <div class="px-6 py-8 rounded-lg border-2 dark:bg-gray-900 dark:border-gray-800">
          <span class="block text-sm dark:text-gray-400">Current Month</span>
          <span class="block text-2xl font-semibold">19 114,07 $US</span>
        </div>
      </div>

      <h2 class="font-semibold text-lg mb-4">Récapitulatif sur l'année</h2>
      <client-only>
        <linear-chart :data="[]" />
      </client-only>

      <c-card>
        <c-card-header title="Alerte budget" />
        <c-card-body>
          plop
        </c-card-body>
      </c-card>

    </div>

    <div class="col-span-4 flex flex-col gap-16">

      <c-select-billing-account v-if="currentBillingAccount" :collections="billingAccounts" v-model="currentBillingAccount" />

      <client-only>
        <pie-chart />
      </client-only>

      <div>
        // plop
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import LinearChart from "~/components/chart/LinearChart.vue";
import PieChart from "~/components/chart/PieChart.vue";
import CCard from "~/components/card/CCard.vue";
import CCardHeader from "~/components/card/CCardHeader.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CSelectBillingAccount from "~/components/forms/select/CSelectBillingAccount.vue";

const billingAccounts = [
  { name: 'Mon compte de facturation' },
  { name: 'Mon compte de facturation 1' },
  { name: 'Mon compte de facturation 2' }
]
const currentBillingAccount = ref(billingAccounts[0])
</script>

<style scoped>

</style>