<template>
  <div>
    <h2 class="text-base font-semibold text-gray-900">{{ props.title }}</h2>
    <p class="mt-1 text-sm text-gray-500">{{ props.description }}</p>
    <ul
      v-if="props.items.length > 0"
      role="list"
      class="mt-6 grid grid-cols-1 gap-6 border-b border-t border-gray-200 py-6 sm:grid-cols-2"
    >
      <li v-for="(item, itemIdx) in items" :key="itemIdx" class="flow-root">
        <div
          class="relative -m-2 flex items-center space-x-4 rounded-xl p-2 focus-within:ring-2 focus-within:ring-indigo-500 hover:bg-gray-50"
        >
          <div
            :class="[
              item.background,
              'flex size-16 shrink-0 items-center justify-center rounded-lg',
            ]"
          >
            <component
              :is="item.logo"
              class="size-6 text-white"
              aria-hidden="true"
            />
          </div>
          <div>
            <h3 class="text-sm font-medium text-gray-900">
              <a href="#" class="focus:outline-none">
                <span class="absolute inset-0" aria-hidden="true" />
                <span>{{ item.title }}</span>
                <span aria-hidden="true"> &rarr;</span>
              </a>
            </h3>
            <p class="mt-1 text-sm text-gray-500">{{ item.description }}</p>
          </div>
        </div>
      </li>
    </ul>
    <div v-if="$slots.default" class="mt-6 pt-6 border-t border-gray-200">
      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup>
interface Props {
  title?: string;
  description?: string;
  documentationUrl?: string;
  items?: Array<{
    title: string;
    description: string;
    background: string;
    logo: string;
    components: any;
  }>;
}

const props = withDefaults(defineProps<Props>(), {
  items: [],
});
</script>
