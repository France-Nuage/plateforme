<template>
  <div class="flex justify-end gap-4">
    <slot>
      <app-button
        size="sm"
        variant="filled"
        v-if="!props.noExport"
        prepend-icon="solar:download-square-bold-duotone"
      >
        Exporter
      </app-button>
      <app-button
        size="sm"
        variant="primary"
        prepend-icon="solar:add-circle-bold-duotone"
        @click="router.push(props.to)"
      >
        {{ props.label }}
      </app-button>
    </slot>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import AppButton from '@/components/forms/app-button.vue'

const router = useRouter()

interface Props {
  noExport?: boolean
  label?: string
  to: string
}

const props = defineProps<Props>()
</script>
