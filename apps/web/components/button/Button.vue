<template>
  <button
    :class="[
      `inline-flex gap-0.5 justify-center overflow-hidden text-sm font-medium transition ${variantClasses[props.variant]}`,
      { 'w-full': props.block },
    ]"
    @click="handleClick"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
interface Props {
  variant?: "primary" | "secondary" | "filled" | "outline" | "text";
  arrow?: "left" | "right";
  to?: string;
  block?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: "primary",
});

const variantClasses = {
  primary: "rounded-lg bg-primary py-3 px-5 text-white hover:bg-primary-700",
  secondary: "rounded-lg bg-white py-3 px-5 text-primary hover:bg-white",
  filled: "rounded-lg bg-zinc-900 py-3 px-5 text-white hover:bg-primary-700",
  outline:
    "rounded-lg py-3 px-5 text-zinc-700 ring-1 ring-inset ring-zinc-900/10 hover:bg-primary-900/2.5 hover:text-zinc-900",
  text: "text-emerald-500 hover:text-emerald-600",
};

// Gestion du clic
const handleClick = () => {
  if (props.to) {
    // Redirection via window.location
    window.location.href = props.to;
  }
};
</script>

<style scoped></style>
