<script setup lang="ts">
import PTable from '@/components/table/PTable.vue'
import { onMounted, ref } from 'vue'
import { list } from '@/services/instance-service'
import type { InstanceInfo } from '@/protocol/controlplane.ts'
import LayoutDefault from '@/components/layouts/LayoutDefault.vue'
import PTableActions from '@/components/table/PTableActions.vue'

const headers = [
  { key: 'select', label: 'Nom' },
  { key: 'id', label: 'Id' },
  { key: 'status', label: 'Status' },
  { key: 'maxCpuCores', label: 'Max CPU' },
  { key: 'cpuUsagePercent', label: 'CPU usage percent' },
  { key: 'maxMemoryBytes', label: 'Max Memory Bytes' },
  { key: 'memoryUsageBytes', label: 'Memory Usage Bytes' },
]

const instances = ref<InstanceInfo[]>([])

onMounted(() => {
  list().then((response: InstanceInfo[]) => {
    instances.value = response
  })
})
</script>

<template>
  <layout-default>
    <h1 class="mb-4">Compute</h1>
    <p-table-actions to="/compute/create" no-export label="Créer une instance" />
    <p-table :headers="headers" :data="instances" name="compute_vm_list" />
  </layout-default>
</template>
