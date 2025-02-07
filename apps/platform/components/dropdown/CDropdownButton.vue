<template>
  <MenuButton class="-m-1.5 flex items-center p-1.5">
    <slot />
  </MenuButton>
</template>

<script setup lang="ts">
import { MenuButton } from "@headlessui/vue";
</script>
