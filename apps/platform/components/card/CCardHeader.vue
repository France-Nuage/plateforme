<template>
  <div class="bg-white border-b border-gray-200 dark:bg-gray-800 dark:border-gray-700 px-4 py-4 rounded-t-md">
    <h2 class="font-semibold dark:text-gray-200">{{ props.title  }}</h2>
    <p class="text-sm" v-if="props.description">{{ props.description  }}</p>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title: string;
  description?: string;
}

const props = defineProps<Props>()
</script>


<style scoped>

</style>