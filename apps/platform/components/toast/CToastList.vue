<template>
  <div
    aria-live="assertive"
    class="pointer-events-none fixed right-5 bottom-5 flex items-end px-4 py-6 sm:items-start sm:p-6"
    style="width: 600px"
  >
    <TransitionGroup
      name="list"
      tag="ul"
      class="flex w-full flex-col items-center space-y-4 sm:items-end"
      enter-active-class="transform ease-out duration-300 transition"
      enter-from-class="translate-y-2 opacity-0 sm:translate-y-0 sm:translate-x-2"
      enter-to-class="translate-y-0 opacity-100 sm:translate-x-0"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <li v-for="(item, key) in alerts.list" :key="key" class="w-full">
        <c-toast-item
          :title="item.title"
          :description="item.description"
          :type="item.type"
        />
      </li>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import CToastItem from "~/components/toast/CToastItem.vue";

const alerts = useAlerts();
</script>
