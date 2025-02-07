<template>
  <tab v-slot="{ selected }" class="outline-none">
    <div
      :class="[
        '-mb-0.5 py-2 px-3 relative text-gray-500 dark:text-gray-400 font-medium text-sm outline-none',
        {
          'border-primary border-b-2 text-primary dark:text-primary': selected,
        },
      ]"
    >
      <slot />
    </div>
  </tab>
</template>

<script setup lang="ts">
import { Tab } from "@headlessui/vue";
</script>
