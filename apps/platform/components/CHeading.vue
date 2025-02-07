<template>
  <div class="flex justify-between border-b border-gray-200 pb-5">
    <div>
      <h2 class="block font-semibold text-3xl">{{ props.title }}</h2>
      <span
        v-if="props.description"
        class="block mt-2 text-sm text-gray-800 max-w-lg"
        >{{ props.description }}</span
      >
    </div>
    <div v-if="!noAction" class="flex gap-4">
      <c-button
        size="sm"
        variant="filled"
        @click="router.push(props.documentationUrl)"
        >Docs</c-button
      >
      <c-button size="sm" variant="success" @click="router.push(props.to)"
        >Ajouter une instance</c-button
      >
    </div>
  </div>
</template>

<script setup lang="ts">
import CButton from "~/components/forms/CButton.vue";

interface Props {
  title: string;
  description: string;
  documentationUrl: string;
  to: string;
  noAction?: boolean;
}

const router = useRouter();
const props = defineProps<Props>();
</script>

<style scoped></style>
