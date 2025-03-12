<script setup lang="ts">
definePageMeta({
  middleware: false,
});

// Return a 200 OK status
useHead({
  title: 'Health Check',
});
</script>

<template>
  <div>Ok!</div>
</template>
