<template>
  <nav
    class="flex justify-between border-b border-gray-200 py-4 px-4 dark:bg-gray-900 dark:border-gray-800"
    aria-label="Breadcrumb"
  >
    <ol role="list" class="flex items-center space-x-4">
      <li v-if="props.logo">
        <div>
          <a href="#" class="text-gray-400 hover:text-gray-500">
            <c-logo class="size-5 shrink-0" aria-hidden="true" />
            <span class="sr-only">Home</span>
          </a>
        </div>
      </li>
      <li v-for="page in props.pages" :key="page.name">
        <div class="flex items-center">
          <svg
            class="size-5 shrink-0 text-gray-300"
            fill="currentColor"
            viewBox="0 0 20 20"
            aria-hidden="true"
          >
            <path d="M5.555 17.776l8-16 .894.448-8 16-.894-.448z" />
          </svg>
          <nuxt-link
            v-if="page.to"
            :to="page.to"
            class="ml-4 text-sm font-medium text-gray-500 hover:text-gray-700"
            :aria-current="page.current ? 'page' : undefined"
            >{{ page.name }}</nuxt-link
          >
          <span
            v-else
            class="ml-4 text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400"
            :aria-current="page.current ? 'page' : undefined"
            >{{ page.name }}</span
          >
        </div>
      </li>
    </ol>

    <div>
      <slot />
    </div>
  </nav>
</template>

<script setup lang="ts">
interface Props {
  pages: Array<{ name: string; current: string; to?: string }>;
  logo: boolean;
}
const props = defineProps<Props>();
</script>
