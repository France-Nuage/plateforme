<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { list } from '@/services/instance-service'

interface Props {
  id: string
}

const instance = ref(null)
const props = defineProps<Props>()

onMounted(() => {})
</script>

<template>
  <div>
    <pre>
      {{ props }}
    </pre>
  </div>
</template>
