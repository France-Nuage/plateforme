<template>
  <transition
    enter-active-class="transition ease-out duration-100"
    enter-from-class="transform opacity-0 scale-95"
    enter-to-class="transform opacity-100 scale-100"
    leave-active-class="transition ease-in duration-75"
    leave-from-class="transform opacity-100 scale-100"
    leave-to-class="transform opacity-0 scale-95"
  >
    <MenuItems
      class="absolute min-w-40 right-0 z-10 mt-2.5 origin-top-right rounded-lg bg-white p-2 shadow-lg ring-1 ring-gray-900/5 focus:outline-none"
    >
      <slot />
    </MenuItems>
  </transition>
</template>

<script setup lang="ts">
import { MenuItems } from "@headlessui/vue";
</script>
