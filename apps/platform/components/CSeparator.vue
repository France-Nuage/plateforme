<template>
  <div class="my-10">
    <div class="relative">
      <div class="absolute inset-0 flex items-center" aria-hidden="true">
        <div
          class="w-full border-t border-gray-200 dark:border-gray-700 rounded-md"
        />
      </div>
      <div class="relative flex justify-center text-sm/6 font-medium">
        <span
          class="bg-white px-6 text-gray-900 dark:bg-gray-800 dark:text-gray-300 rounded-md"
          >{{ props.label }}</span
        >
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  label: string;
}

const props = defineProps<Props>();
</script>

<style scoped></style>
