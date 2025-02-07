<template>
  <template>
    <div :class="[`${variantClasses[props.variant].background} sm:rounded-lg`]">
      <div class="px-4 py-5 sm:p-6">
        <h3 class="text-base font-semibold text-gray-900">{{ props.title }}</h3>
        <div
          v-if="props.description"
          class="mt-2 max-w-xl text-sm text-gray-500"
        >
          <p>{{ props.description }}</p>
        </div>
        <div class="mt-5">
          <button
            type="button"
            class="inline-flex items-cent er rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50"
          >
            Contact sales
          </button>
        </div>
      </div>
    </div>
  </template>
</template>

<script setup lang="ts">
interface Props {
  title: string;
  description: string;
  variant: "filled" | "success" | "danger" | "warning" | "information";
}

const props = withDefaults(defineProps<Props>(), {
  variant: "information",
});

const variantClasses = {
  information: {
    background: "bg-blue-50",
  },
  danger: {
    background: "bg-red-50",
  },
  warning: {
    background: "bg-orange-50",
  },
  success: {
    background: "bg-green-50",
  },
};
</script>

<style scoped></style>
