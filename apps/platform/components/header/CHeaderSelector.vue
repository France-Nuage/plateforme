<template>
  <c-dropdown>
    <c-dropdown-button class="w-full dark:hover:bg-gray-800 transition-all rounded-lg">
      <div class="px-2">
<!--        <small class="block dark:text-gray-400 text-left">{{ props.type }}</small>-->
        <div class="flex items-center gap-4">
          <span class="text-gray-200 text-sm flex-shrink-0">{{ props.selected?.name || props.noSelectedMessage }}</span>
          <ChevronUpDownIcon
            class="h-5 w-5 text-gray-400 flex-shrink-0"
            aria-hidden="true"
          />
        </div>
      </div>
    </c-dropdown-button>
    <c-dropdown-list style="left: 0; min-width: 200px">

      <div class="flex items-center gap-2 w-full">
        <svg viewBox="0 0 20 20" fill="none" aria-hidden="true" class="h-5 w-5 stroke-current">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12.01 12a4.25 4.25 0 1 0-6.02-6 4.25 4.25 0 0 0 6.02 6Zm0 0 3.24 3.25"
          />
        </svg>
        <input class="outline-none focus:ring-0 border-none w-full text-sm p-0" placeholder="Rechercher" v-model="searchTerms" />
      </div>

      <c-dropdown-divider />

      <template v-if="filteredItems.length > 0">
        <c-dropdown-item v-for="item in filteredItems" @click="$emit('select', item.value)">{{ item.name }}</c-dropdown-item>
      </template>
      <div v-else class="text-center">
        <small class="text-gray-400">Aucun résultat</small>
      </div>

      <c-dropdown-divider />

      <div>
        <c-dropdown-item class="flex-shrink-0 text-sm text-primary" @click="$emit('add')">+ Ajouter un projet</c-dropdown-item>
      </div>
    </c-dropdown-list>
  </c-dropdown>
</template>

<script setup lang="ts">

import CDropdown from "~/components/dropdown/CDropdown.vue";
import CDropdownButton from "~/components/dropdown/CDropdownButton.vue";
import CDropdownList from "~/components/dropdown/CDropdownList.vue";
import CDropdownItem from "~/components/dropdown/CDropdownItem.vue";
import { ChevronUpDownIcon } from "@heroicons/vue/20/solid";
import CDropdownDivider from "~/components/dropdown/CDropdownDivider.vue";

interface Props {
  type: 'project' | 'folder';
  items: Array<{ name: string; value: string }>;
  selected?: { name: string; value: string } | null;
  noSelectedMessage?: string;
}

const props = withDefaults(defineProps<Props>(), {
  noSelectedMessage: 'Aucun selectionné'
})

const searchTerms = ref('')
const filteredItems = computed(() => {
  const search = searchTerms.value.trim().toLowerCase();
  return search.length ? props.items.filter((item) => item.name.toLowerCase().trim().includes(search)) : props.items;
})
</script>
