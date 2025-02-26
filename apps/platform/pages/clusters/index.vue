<template>
  <nuxt-layout>
    <p>mes super clusters</p>
  </nuxt-layout>
</template>

<script setup lang="ts">
const { servers } = storeToRefs(useServerStore());
</script>

<style scoped></style>
