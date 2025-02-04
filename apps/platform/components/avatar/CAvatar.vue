<template>
  <span class="relative inline-block">
    <img
      :class="[`rounded-full bg-gray-50 ${sizeClasses[props.size]}`]"
      src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
      alt=""
    />
    <span
      v-if="props.dotted"
      :class="[
        `absolute bottom-0 right-0 block size-4 rounded-full bg-green-400 ring-2 ring-white ${sizeDottedClasses[props.size]}`,
      ]"
    />
  </span>
</template>

<script setup lang="ts">
interface Props {
  size?: "xs" | "sm" | "md" | "lg" | "xl" | "xxl";
  dotted?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: "md",
});

const sizeClasses = {
  xs: "size-6",
  sm: "size-8",
  md: "size-10",
  lg: "size-12",
  xl: "size-14",
  xxl: "size-16",
};

const sizeDottedClasses = {
  xs: "size-1.5",
  sm: "size-2",
  md: "size-2.5",
  lg: "size-3",
  xl: "size-3.5",
  xxl: "size-4",
};
</script>
