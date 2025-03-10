<template>
  <div class="bg-gray-900 py-24 sm:py-32 -mt-20">
    <div class="mx-auto max-w-7xl px-6 lg:px-8">
      <dl class="grid grid-cols-1 gap-x-8 gap-y-16 text-center lg:grid-cols-4">
        <div
          v-for="stat in stats"
          :key="stat.id"
          class="mx-auto flex max-w-xs flex-col gap-y-4"
        >
          <dt class="text-base/7 text-gray-400">{{ stat.name }}</dt>
          <dd
            class="order-first text-3xl font-semibold tracking-tight text-white sm:text-5xl"
          >
            {{ stat.value }}
          </dd>
        </div>
      </dl>
    </div>
  </div>
</template>

<script setup>
const stats = [
  { id: 1, name: "Support", value: "24/7" },
  { id: 2, name: "SLA", value: "99,9%" },
  { id: 3, name: "Coût potentiel", value: "-60%" },
  { id: 4, name: "Souveraineté", value: "100%" },
];
</script>
