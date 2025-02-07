<template>
  <div :class="[`rounded-md ${variantClasses[props.variant].background} p-4`]">
    <div class="flex">
      <div class="shrink-0">
        <XCircleIcon
          v-if="variant === 'danger'"
          class="size-5 text-red-400"
          aria-hidden="true"
        />
        <InformationCircleIcon
          v-if="variant === 'information'"
          class="size-5 text-blue-400"
          aria-hidden="true"
        />
      </div>
      <div class="ml-3">
        <h3
          v-if="props.title"
          :class="[
            `text-sm font-medium ${variantClasses[props.variant].title}`,
          ]"
        >
          {{ props.title }}
        </h3>
        <div
          :class="[`mt-2 text-sm ${variantClasses[props.variant].description}`]"
        >
          <div class="space-y-1">
            <slot />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { XCircleIcon, InformationCircleIcon } from "@heroicons/vue/20/solid";

interface Props {
  title: string;
  variant?: "filled" | "success" | "danger" | "warning" | "information";
}

const props = withDefaults(defineProps<Props>(), { variant: "information" });

const variantClasses = {
  information: {
    background: "bg-blue-50 dark:bg-blue-950",
    title: "text-blue-800 dark:text-white",
    description: "text-blue-600 dark:text-gray-400",
    icon: "text-blue-400",
  },
  danger: {
    background: "bg-red-50 dark:bg-red-950",
    title: "text-red-800 dark:text-white",
    description: "text-red-700 dark:text-gray-400",
    icon: "text-red-400",
  },
  warning: {
    background: "bg-orange-50",
    title: "text-red-800",
    description: "text-red-700",
    icon: "text-red-400",
  },
  success: {
    background: "bg-green-50",
    title: "text-red-800",
    description: "text-red-700",
    icon: "text-red-400",
  },
};
</script>
