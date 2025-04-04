<template>
  <button
    :class="[
      `inline-flex cursor-pointer items-center gap-0.5 justify-center overflow-hidden font-medium transition ${variantClasses[props.variant]} ${sizeClasses[props.size]}`,
      {

        'w-full': props.block,
        'cursor-not-allowed': props.disabled || props.loading,
      },
    ]"
    :disabled="props.disabled || props.loading"
    :type="props.type"
  >
    <svg
      v-if="props.loading"
      class="animate-spin -ml-1 mr-3 h-4 w-4 text-white"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle
        class="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        stroke-width="4"
      ></circle>
      <path
        class="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      ></path>
    </svg>
    <!--    <icon v-else :name="props.prependIcon" v-if="props.prependIcon && !props.loading" class="h-[24px] w-[24px]" />-->
    <slot />
    <!--    <icon :name="props.appendIcon" v-if="props.appendIcon" />-->
  </button>
</template>

<script setup lang="ts">
interface Props {
  variant?:
    | 'primary'
    | 'secondary'
    | 'success'
    | 'filled'
    | 'outline'
    | 'text'
    | 'danger'
    | 'information'
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  arrow?: 'left' | 'right'
  to?: string
  block?: boolean
  loading?: boolean
  disabled?: boolean
  type?: 'submit' | 'button'
  prependIcon?: string
  appendIcon?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'lg',
  type: 'button',
})

const variantClasses = {
  primary:
    'text-black bg-primary hover:bg-primary-bold border border-primary focus:ring-4 focus:ring-primary focus:ring-opacity-50',
  secondary:
    'text-black bg-secondary hover:bg-secondary-bold border border-secondary focus:ring-4 focus:ring-purple-400 focus:ring-opacity-50',
  danger: 'text-white bg-red-600 hover:bg-red-700',
  filled:
    'text-primary bg-gray-800 hover:bg-gray-50 border border-gray-800 focus:ring-4 focus:ring-purple-400 focus:ring-opacity-50',
}

const sizeClasses = {
  xs: 'font-semibold px-2 text-xs h-[29px] rounded-md',
  sm: 'font-semibold px-3 text-sm h-[38px] rounded-lg',
  lg: 'font-semibold px-4 py-3 h-fit rounded-lg',
}
</script>
