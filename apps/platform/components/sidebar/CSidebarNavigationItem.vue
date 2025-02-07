<template>
  <nuxt-link
    :to="props.to"
    :class="[
      'px-2 py-1.5 w-full cursor-pointer inline-flex items-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 hover:shadow-sm justify-start',
      {
        'bg-primary text-white hover:bg-primary': props.active,
        'hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800':
          !props.active,
      },
    ]"
  >
    <Icon :name="`solar:${props.icon}`" size="1.3rem" style="color: #c1c1c1" />
    <span>{{ props.title }}</span>
  </nuxt-link>
</template>

<script setup lang="ts">
interface Props {
  title: string;
  active?: boolean;
  icon?: string;
  to: string;
}

const props = defineProps<Props>();
</script>
