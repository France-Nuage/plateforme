<template>
  <div>
    <h1>{{ error.statusCode }}</h1>
    <div v-html="error?.stack" />
    <NuxtLink to="/">Go back home</NuxtLink>
  </div>
</template>

<script setup lang="ts">
import type { NuxtError } from "#app";

const props = defineProps({
  error: Object as () => NuxtError,
});
</script>
