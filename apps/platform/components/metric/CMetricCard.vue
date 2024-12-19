<template>
  <div class="space-y-4 py-4 md:border-r md:px-4 border-b dark:border-gray-800">
    <div class="flex items-center justify-between">
      <a href="" class="block w-full group">
        <div class="group flex items-center gap-1">
          <p class="text-sm text-foreground-light group-hover:text-foreground transition cursor-pointer">{{ props.title }}</p>
          <span class="transition inline-block group-hover:transform group-hover:translate-x-0.5">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-right transition">
              <path d="m9 18 6-6-6-6"></path>
            </svg>
          </span>
        </div>
        <span class="text-sm">{{ props.partialValue }} / {{ props.totalValue }} {{ props.unit }}</span>
      </a>
      <c-circular-gauge :total-value="props.totalValue" :partial-value="props.partialValue" />
    </div>
  </div>
</template>

<script setup lang="ts">
import CCircularGauge from "~/components/gauge/CCircularGauge.vue";

interface Props {
  title: string;
  totalValue: number;
  partialValue: number;
  unit?: string;
}

const props = defineProps<Props>();
</script>
