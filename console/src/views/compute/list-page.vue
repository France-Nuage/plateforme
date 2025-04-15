<script setup lang="ts">
import AppTable from '@/components/table/app-table.vue'
import { onMounted, ref } from 'vue'
import { create, hypervisors, list } from '@/services/instance-service'
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
  hypervisors.listHypervisors({}).response.then((data) => console.log('result', data))
  list().then((response: InstanceInfo[]) => {
    instances.value = response
  })
})

const createInstance = () => {
  create({
    cpuCores: 1,
    image: 'debian-12-genericcloud-amd64-20250316-2053.qcow2',
    memoryBytes: BigInt(536870912),
    name: 'ACME-missile-guiding-system',
    snippet: 'base-snippet.yaml',
  })
    .then(({ id }) => { console.log(`instance #${id} created`) })
    .catch((error) => console.error("problem", error))
  console.log("creating...");
}

</script>

<template>
  <layout-default>
    <h1 class="mb-4">Compute</h1>
    <button class="bg-indigo-100 border px-4 py-2 rounded" @click="createInstance">Nouvelle instance</button>
    <app-table :headers="headers" :data="instances" name="compute_vm_list" />
  </layout-default>
</template>
