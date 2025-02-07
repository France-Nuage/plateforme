<template>
  <div class="hidden lg:block lg:max-w-md lg:flex-auto">
    <button
      type="button"
      class="hidden h-8 w-full items-center gap-2 rounded-md bg-gray-100 pl-2 pr-3 text-sm text-zinc-500 ring-1 ring-zinc-900/10 transition hover:ring-zinc-900/20 ui-not-focus-visible:outline-none lg:flex dark:bg-white/5 dark:text-zinc-400 dark:ring-inset dark:ring-white/10 dark:hover:ring-white/20"
      @click="searchDialogIsOpen = true"
    >
      <svg
        viewBox="0 0 20 20"
        fill="none"
        aria-hidden="true"
        class="h-5 w-5 stroke-current"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M12.01 12a4.25 4.25 0 1 0-6.02-6 4.25 4.25 0 0 0 6.02 6Zm0 0 3.24 3.25"
        />
      </svg>
      Rechercher quelque chose...
      <kbd class="ml-auto text-2xs text-zinc-400 dark:text-zinc-500">
        <kbd class="font-sans">{{ modifierKey }}</kbd>
        <kbd class="font-sans">K</kbd>
      </kbd>
    </button>

    <c-search-dialog v-model="searchDialogIsOpen" />
  </div>
</template>

<script setup lang="ts">
import SearchDialog from "~/components/search/CSearchDialog.vue";
import CSearchDialog from "~/components/search/CSearchDialog.vue";

const modifierKey = ref("");

onMounted(() => {
  if (import.meta.browser) {
    modifierKey.value = /(Mac|iPhone|iPod|iPad)/i.test(navigator.platform)
      ? "⌘"
      : "Ctrl ";
  }
});

const searchDialogIsOpen = ref(false);
</script>
