<template>
  <MenuItem v-slot="{ active }">
    <div :class="[active ? 'bg-gray-100 rounded-lg relative outline-none' : '', 'flex items-center gap-2 px-3 py-1 text-sm/6 text-gray-900 cursor-pointer']">
      <Icon
        v-if="props.icon"
        class="block"
        :name="props.icon"
        size="24"
      />
      <span class="block">
        <slot />
      </span>
    </div>
  </MenuItem>
</template>

<script setup lang="ts">
import { MenuItem } from '@headlessui/vue'
interface Props {
  icon?: string;
}

const props = defineProps<Props>()
</script>

<style scoped>

</style>