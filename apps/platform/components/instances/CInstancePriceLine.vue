<template>
  <div
    :class="[
      props.active ? 'boring-primary ring-2 ring-primary' : 'border-gray-300',
      'relative block cursor-pointer rounded-lg border bg-white px-6 py-4 shadow-sm focus:outline-none sm:flex sm:justify-between',
    ]"
  >
    <span class="flex items-center">
      <span class="flex flex-col text-sm">
        <span class="font-medium text-gray-900">{{ props.plan.name }}</span>
        <span class="text-gray-500">
          <span class="block sm:inline"
            >{{ props.plan.ram }} / {{ props.plan.cpus }}</span
          >
          {{ " " }}
          <span class="hidden sm:mx-1 sm:inline" aria-hidden="true"
            >&middot;</span
          >
          {{ " " }}
          <span class="block sm:inline">{{ props.plan.disk }}</span>
        </span>
      </span>
    </span>
    <span class="mt-2 flex text-sm sm:ml-4 sm:mt-0 sm:text-right">
      <span class="font-medium text-gray-900">{{ props.plan.price }}</span>
      <span class="ml-1 text-gray-500 sm:ml-0">/mo</span>
    </span>
    <span
      :class="[
        active ? 'border' : 'border-2',
        checked ? 'boring-primary' : 'border-transparent',
        'pointer-events-none absolute -inset-px rounded-lg',
      ]"
      aria-hidden="true"
    />
  </div>
</template>

<script setup lang="ts">
interface Props {
  active?: boolean;
  plan: {
    name: string;
    ram: string;
    cpus: string;
    disk: string;
    price: number;
    currency: string;
  };
}

const props = defineProps<Props>();
</script>

<style scoped></style>
