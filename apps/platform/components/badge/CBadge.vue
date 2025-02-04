<template>
  <span
    :class="[
      props.pill ? 'rounded-full' : 'rounded-md',
      props.flat ? '' : 'ring-1 ring-inset ring-gray-500/10',
      `inline-flex items-center ${variantClasses[props.variant].background} ${variantClasses[props.variant].title} px-1.5 py-0.5 text-xs font-medium gap-2`,
      {},
    ]"
  >
    <svg
      v-if="props.dotted"
      :class="[`size-1.5`, {}]"
      viewBox="0 0 6 6"
      aria-hidden="true"
    >
      <circle cx="3" cy="3" r="3" />
    </svg>

    <slot />

    <button
      v-if="props.remove"
      type="button"
      :class="[
        `group relative -mr-1 size-3.5 rounded-sm hover:${variantClasses[props.variant].background}/20`,
      ]"
      @click.stop="emit('remove', $event)"
    >
      <span class="sr-only">Remove</span>
      <svg
        viewBox="0 0 14 14"
        :class="[
          `size-3.5 ${variantClasses[props.variant].svg} group-hover:${variantClasses[props.variant].svg}/75`,
        ]"
      >
        <path d="M4 4l6 6m0-6l-6 6" />
      </svg>
      <span class="absolute -inset-1" />
    </button>
  </span>
</template>

<script setup lang="ts">
interface Props {
  dotted?: boolean;
  pill?: boolean;
  remove?: boolean;
  flat?: boolean;
  small?: boolean;
  variant?:
    | "filled"
    | "success"
    | "danger"
    | "warning"
    | "information"
    | "default";
}

const props = withDefaults(defineProps<Props>(), {
  variant: "default",
});
const emit = defineEmits(["remove"]);

const variantClasses = {
  default: {
    svg: "stroke-gray-600",
    background: "bg-gray-100",
    title: "text-gray-600",
  },
  information: {
    svg: "stroke-blue-800 dark:stroke-blue-300",
    background: "bg-blue-50 fill-blue-300 dark:bg-blue-950",
    title: "text-blue-800 dark:text-blue-300",
  },
  danger: {
    svg: "stroke-red-800",
    background: "bg-red-50 dark:bg-red-950",
    title: "text-red-800 dark:text-white",
  },
  warning: {
    svg: "stroke-red-800",
    background: "bg-orange-50",
    title: "text-red-800",
  },
  success: {
    svg: "stroke-red-800",
    background: "bg-green-50",
    title: "text-red-800",
  },
};
</script>
