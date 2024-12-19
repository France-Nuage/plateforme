<template>
  <nuxt-layout>
    <h1 class="mb-4">Plan & facturation</h1>
    <TabGroup>
      <c-tab-list>
        <c-tab>Facturation</c-tab>
        <c-tab>Usages</c-tab>
      </c-tab-list>
      <TabPanels>
        <TabPanel><Billings /></TabPanel>
        <TabPanel><Quotas /></TabPanel>
      </TabPanels>
    </TabGroup>
  </nuxt-layout>
</template>

<script setup lang="ts">
import CTabList from "~/components/tabs/CTabList.vue";
import CTab from "~/components/tabs/CTab.vue";
import { TabGroup, TabPanels, TabPanel } from '@headlessui/vue'
import Quotas from "~/pages/billings/local-components/Quotas.vue";
import Billings from "~/pages/billings/local-components/Billings.vue";
</script>
