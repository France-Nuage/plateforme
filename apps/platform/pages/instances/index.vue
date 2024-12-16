<template>
  <nuxt-layout>
    <div>
      <empty-screen />
    </div>
  </nuxt-layout>
</template>

<script setup lang="ts">
import EmptyScreen from "~/pages/instances/local-components/empty-screen.vue";

const { servers } = storeToRefs(useServerStore());
// const { loadServers } = useServerStore();

onMounted(() => {
  // loadServers();
  // setInterval(() => {
  //   loadServers();
  // }, 1000);
})
</script>

<style scoped>

</style>