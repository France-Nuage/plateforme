<template>
  <TabList
    class="border-b border-gray-200 dark:border-gray-800 -mx-8 px-8 mb-8"
  >
    <slot />
  </TabList>
</template>

<script setup lang="ts">
import { TabList } from "@headlessui/vue";
</script>

<style scoped></style>
