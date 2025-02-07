<template>
  <div
    class="flex flex-col gap-2 border-b border-gray-200 dark:border-gray-800 -mx-4 px-4 py-4"
  >
    <span
      v-if="props.title"
      class="text-sm text-gray-500 dark:text-gray-500 mb-2"
      >{{ props.title }}</span
    >
    <slot />

    <slot name="footer" />
  </div>
</template>

<script setup lang="ts">
interface Props {
  title?: string;
}

const props = defineProps<Props>();
</script>

<style scoped></style>
