<template>
  <li
    :class="[
      'group block px-4 py-3 aria-selected:bg-zinc-50 dark:aria-selected:bg-zinc-800/50 cursor-pointer',
      {
        'border-t border-zinc-100 dark:border-zinc-800': props.key > 0,
      },
    ]"
    :aria-labelledby="`${props.key}-hierarchy ${props.key}-title`"
    @click="$router.push(props.collection.href)"
    role="listitem"
  >
    <div
      :id="`${props.key}-title`"
      aria-hidden="true"
      class="text-sm font-medium text-zinc-900 group-aria-selected:text-emerald-500 dark:text-white"
      role="button"
    >
      <c-search-highlight-text
        :query="props.query"
        :text="props.collection.name"
        class="text-sm font-medium text-zinc-900 group-aria-selected:text-emerald-500 dark:text-white"
      />
      <c-search-highlight-text
        :query="props.query"
        :text="props.collection.path"
        class="mt-1 truncate whitespace-nowrap text-2xs text-zinc-500"
      />
    </div>
  </li>
</template>

<script setup lang="ts">
import CSearchHighlightText from "~/components/search/CSearchHighlightText.vue";

interface Props {
  collection: any;
  key: number;
  query: string;
}

const props = defineProps<Props>();
</script>

<style scoped></style>
