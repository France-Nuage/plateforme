<template>
  <nuxt-layout>

<!--    <div v-if="servers.length > 0" class="flex flex-col gap-4">-->
<!--      <div class="shadow-sm bg-white border border-gray-200 px-4 py-4 rounded-lg flex gap-4" v-for="(server, key) in servers" :key="key">-->
<!--        <div>{{ server.status }}</div>-->
<!--        <div>{{ server.name }}</div>-->
<!--      </div>-->
<!--    </div>-->
<!--    <div v-else>Aucun serveur pour le moment</div>-->
<!---->
<!--    <c-button class="mt-8" @click="createServer({ name: '' })" size="sm">Ajouter un serveur</c-button>-->
    Bienvenue sur France-Nuage
  </nuxt-layout>
</template>

<script setup lang="ts">
import CButton from "~/components/forms/CButton.vue";

const { servers } = storeToRefs(useServerStore());
// const { loadServers, createServer } = useServerStore();

// onMounted(() => {
//   loadServers();
//   setInterval(() => {
//     loadServers();
//   }, 1000);
// })
</script>

<style scoped>

</style>