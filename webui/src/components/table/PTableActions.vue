<template>
  <div class="flex justify-end gap-4">
    <slot>
      <b-button
        size="sm"
        variant="filled"
        v-if="!props.noExport"
        prepend-icon="solar:download-square-bold-duotone"
      >
        Exporter
      </b-button>
      <b-button
        size="sm"
        variant="primary"
        prepend-icon="solar:add-circle-bold-duotone"
        @click="router.push(props.to)"
      >
        {{ props.label }}
      </b-button>
    </slot>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import BButton from '@/components/forms/PButton.vue'

const router = useRouter()

interface Props {
  noExport?: boolean
  label?: string
  to: string
}

const props = defineProps<Props>()
</script>
