<template>
  <div class="px-4 pt-6 sm:px-6">
    <div class="flex items-center justify-between">
      <DialogTitle class="text-lg font-semibold text-white">{{
        props.title
      }}</DialogTitle>
    </div>
    <div v-if="props.description" class="mt-2">
      <p class="text-sm text-gray-400 leading-6" v-html="props.description" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { DialogTitle } from "@headlessui/vue";

interface Props {
  title: string;
  description?: string;
}

const props = defineProps<Props>();
</script>
