<template>
  <div>
    <svg class="h-8 w-8 -rotate-90 transform">
      <circle cx="15" cy="15" r="12" fill="transparent" stroke="currentColor" stroke-width="4" class="text-gray-700"></circle>
      <circle cx="15" cy="15" r="12" fill="transparent" stroke="currentColor" stroke-dasharray="75.398" :stroke-dashoffset="`calc(75.39822 - ${percent} / 100 * 75.39822)`" stroke-width="4" class="text-gray-200"></circle>
    </svg>
  </div>
</template>

<script setup lang="ts">
interface Props {
  totalValue: number;
  partialValue: number;
}

const props = withDefaults(defineProps<Props>(), {
  totalValue: 100,
  partialValue: 0
})

const percent =  computed(() => {
  return  (100 * props.partialValue) / props.totalValue
})
</script>
