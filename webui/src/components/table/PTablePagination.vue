<template>
  <div class="flex justify-between items-center">
    <div class="lead">
      {{ props.current * props.limit }} résultats sur environ {{ props.total }} résultats
    </div>
    <div class="flex gap-4">
      <b-button size="sm" variant="filled" @click="emit('previous')">Précédent</b-button>
      <b-button size="sm" variant="filled" @click="emit('next')">Suivant</b-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import BButton from '@/components/forms/PButton.vue'

interface Props {
  total: number
  limit: number
  current: number
}

const props = withDefaults(defineProps<Props>(), { total: 0, current: 1, limit: 1 })

const emit = defineEmits(['next', 'previous'])
</script>
