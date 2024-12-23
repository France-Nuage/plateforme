<template>
  <nuxt-layout>
    <div>
      <div v-if="instance">
        <h1>Instance ({{ instance.name }})</h1>
      </div>
    </div>
  </nuxt-layout>
</template>

<script setup lang="ts">
const { instance } = storeToRefs(useInstanceStore());
const { loadInstance } = useInstanceStore();
const route = useRoute()

onMounted(() => {
  loadInstance(route.params.id);
})
</script>
