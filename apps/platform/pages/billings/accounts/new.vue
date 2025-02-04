<template>
  <div class="bg-gray-50 h-screen dark:bg-gray-900">
    <c-breadcrumb logo :pages="pages" />
    <nuxt-page />
  </div>
</template>

<script setup lang="ts">
import CBreadcrumb from "~/components/breadcrumb/CBreadcrumb.vue";
const pages = ref([
  { name: "Information", href: "#", current: true },
  { name: "Payment", href: "#", current: false },
]);
</script>
