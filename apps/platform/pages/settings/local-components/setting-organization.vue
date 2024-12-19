<template>
  <h1 class="text-3xl font-semibold mb-8 dark:text-gray-300">{{ organization.name }}</h1>

  <TabGroup>
    <c-tab-list>
      <c-tab>Général</c-tab>
<!--      <c-tab>Facturation</c-tab>-->
    </c-tab-list>
    <TabPanels>
      <TabPanel class="w-6/12">
        <general />
      </TabPanel>
<!--      <TabPanel>-->
<!--        <billing />-->
<!--      </TabPanel>-->
    </TabPanels>
  </TabGroup>
</template>

<script setup lang="ts">
import { TabGroup, TabPanels, TabPanel } from '@headlessui/vue'
import CTabList from "~/components/tabs/CTabList.vue";
import CTab from "~/components/tabs/CTab.vue";
import General from "~/pages/settings/organizations/[id]/local-components/general.vue";

const { organization } = storeToRefs(useNavigationStore())
</script>
