<template>
  <div class="top-0 z-10   flex h-16 shrink-0 items-center gap-x-4 border-b border-gray-200 bg-white dark:border-gray-800 dark:bg-gray-900 px-4 sm:gap-x-6 sm:px-6 lg:px-8">
    <button type="button" class="-m-2.5 p-2.5 text-gray-700 lg:hidden" @click="sidebarOpen = true">
      <span class="sr-only">Open sidebar</span>
      <Bars3Icon class="size-6" aria-hidden="true" />
    </button>

    <!-- Separator -->
    <div class="h-6 w-px bg-gray-900/10 lg:hidden" aria-hidden="true" />

    <div class="flex flex-1 gap-x-4 justify-between lg:gap-x-6 relative">
      <div class="flex items-center gap-4">
        <c-header-selector-folder />
        <svg class="size-5 shrink-0 text-gray-300" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
          <path d="M5.555 17.776l8-16 .894.448-8 16-.894-.448z" />
        </svg>
        <c-header-selector-project />
      </div>
      <div class="relative flex items-center max-w-96 w-full">
        <c-search />
      </div>
      <div class="flex items-center gap-x-4 lg:gap-x-6">
<!--        <button type="button" class="-m-2.5 p-2.5 text-gray-400 hover:text-gray-500">-->
<!--          <span class="sr-only">View notifications</span>-->
<!--          <BellIcon class="size-6" aria-hidden="true" />-->
<!--        </button>-->

        <!-- Separator -->
        <div class="hidden lg:block lg:h-6 lg:w-px lg:bg-gray-900/10" aria-hidden="true" />

        <c-header-profile />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {ref} from "vue";
import {
  Bars3Icon,
  BellIcon,
} from '@heroicons/vue/24/outline'
import CSearch from "~/components/search/CSearch.vue";
import CHeaderProfile from "~/components/header/CHeaderProfile.vue";
import CHeaderSelectorFolder from "~/components/header/CHeaderSelectorFolder.vue";
import CHeaderSelectorProject from "~/components/header/CHeaderSelectorProject.vue";

const sidebarOpen = ref(false)
</script>
