<template>
  <div class="hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex">
    <div
      class="lg:flex lg:w-72 lg:flex-col border-r border-gray-200 bg-white dark:bg-gray-900 dark:border-gray-800"
    >
      <div
        class="h-16 border-b border-gray-200 dark:border-gray-800 flex items-center"
      >
        <c-sidebar-resource-selector />
      </div>

      <div class="px-4">
        <nav>
          <slot />
        </nav>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import CSidebarResourceSelector from "~/components/sidebar/CSidebarResourceSelector.vue";
</script>

<style scoped></style>
