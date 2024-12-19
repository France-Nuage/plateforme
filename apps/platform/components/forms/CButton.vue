<template>
  <button
      :class="[`inline-flex items-center h-fit gap-0.5 justify-center overflow-hidden font-medium transition ${variantClasses[props.variant]} ${sizeClasses[props.size]}`, {
        'w-full': props.block,
        'cursor-not-allowed': props.disabled || props.loading
      }]"
      :disabled="props.disabled || props.loading"
  >
    <svg v-if="props.loading" class="animate-spin -ml-1 mr-3 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
    <slot />
  </button>
</template>

<script setup lang="ts">
interface Props {
  variant?: 'primary' | 'secondary' | 'success' | 'filled' | 'outline' | 'text' | 'danger' | 'information';
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  arrow?: 'left' | 'right';
  to?: string;
  block?: boolean;
  loading?: boolean;
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
})

const variantClasses = {
  primary:
      'rounded-lg bg-primary text-white border border-transparent hover:bg-primary-700',
  secondary:
      'rounded-lg bg-white text-primary border-transparent hover:bg-white',
  success:
      'rounded-lg text-white bg-green-600 border border-green-700 hover:bg-green-700',
  danger:
      'rounded-lg text-white bg-red-600 border border-red-700 hover:bg-red-700',
  filled:
      'rounded-lg bg-zinc-100 border border-gray-200 text-black hover:bg-primary-700 dark:border-gray-500 dark:bg-gray-500 dark:text-gray-100',
  outline:
      'rounded-lg text-zinc-700 ring-1 ring-inset ring-zinc-900/10 border-transparent hover:bg-primary-900/2.5 hover:text-zinc-900',
  text: 'text-emerald-500 hover:text-emerald-600',
  information: 'rounded-lg bg-blue-600 border border-blue-700 hover:bg-blue-700 text-white',
}

const sizeClasses = {
  xs: 'xs',
  sm: 'text-xs py-2 px-3',
  md: 'text-sm py-3 px-5',
  lg: 'text-sm py-3 px-5',
  xl: 'text-sm py-3 px-5',
}
</script>

<style scoped>

</style>