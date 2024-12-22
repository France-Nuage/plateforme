<template>
  <nuxt-layout>
    <div>
      <div v-if="instances.length">

        <c-table name="instances" :data="instances" />

      </div>
      <empty-screen v-else />
    </div>
  </nuxt-layout>
</template>

<script setup lang="ts">
import EmptyScreen from "~/pages/instances/local-components/empty-screen.vue";
import CTable from "~/components/table/CTable.vue";

const { instances } = storeToRefs(useInstanceStore());
const { loadInstances } = useInstanceStore();
const interval = ref();

onMounted(() => {
  loadInstances();
  interval.value = setInterval(() => {
    loadInstances();
  }, 5000);
})

onUnmounted(() => {
  clearInterval(interval.value);
})
</script>

<style scoped>

</style>