<script setup lang="ts">
import AppTable from '@/components/table/app-table.vue'
import { onMounted, ref } from 'vue'
import { list } from '@/services/instance-service'
import type { InstanceInfo } from '@/protocol/instances'
import LayoutDefault from '@/components/layouts/layout-default.vue'

const headers = [
  { key: 'id', label: 'Id' },
  { key: 'name', label: 'Name' },
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
    <app-table :headers="headers" :data="instances" name="compute_vm_list" />
  </layout-default>
</template>
