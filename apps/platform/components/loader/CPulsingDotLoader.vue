<template>
  <div class="relative inline-flex">
    <div :class="[`w-3 h-3 ${bgVariants[props.variant]} rounded-full`]"></div>
    <div v-if="props.pulsing" :class="[`w-3 h-3 ${bgVariants[props.variant]} rounded-full absolute top-0 left-0 animate-ping`]"></div>
    <div v-if="props.pulsing" :class="[`w-3 h-3 ${bgVariants[props.variant]} rounded-full absolute top-0 left-0 animate-pulse`]"></div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  variant?: 'success' | 'warning' | 'error' | 'info';
  pulsing?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'info'
})
const bgVariants = {
  success: 'bg-green-500',
  warning: 'bg-yellow-500',
  error: 'bg-red-500',
  info: 'bg-blue-500'
}
</script>
