<template>
  <div class="flex gap-4 px-4 py-4 justify-end">
    <c-button size="sm" variant="filled" @click="emit('cancel')">Annuler</c-button>
    <c-button size="sm" variant="success" @click="emit('add')" :loading="props.loading">Ajouter</c-button>
  </div>
</template>

<script setup lang="ts">
import CButton from "~/components/forms/CButton.vue";

interface Props {
  loading?: boolean;
}

const props = defineProps<Props>()
const emit = defineEmits(['cancel', 'add'])
</script>

<style scoped>

</style>