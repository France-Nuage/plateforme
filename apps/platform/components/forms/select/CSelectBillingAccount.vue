<template>
  <Listbox v-model="selectedItem" v-if="selectedItem">
    <div class="relative">
      <ListboxButton class="relative h-full text-start px-5 py-5 flex gap-4 w-full justify-between rounded-md border-0 shadow-sm ring-1 ring-inset sm:text-sm/6 ring-gray-300 placeholder:text-gray-400 dark:ring-gray-800 h-full">
        <div>
          <span class="text-sm block dark:text-gray-500">Nom du compte de facturation</span>
          <span class="text-lg font-semibold">{{ selectedItem.name }}</span>
        </div>
        <div>
          <Icon :name="`solar:round-double-alt-arrow-down-bold-duotone`" size="32px" class="text-gray-400" />
        </div>
      </ListboxButton>
      <transition
          leave-active-class="transition duration-100 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
      >
        <ListboxOptions
          class="absolute mt-1 z-10 max-h-60 w-full overflow-auto rounded-md bg-white dark:bg-gray-900 dark:border dark:border-gray-800 text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm"
        >
          <ListboxOption
            v-for="collection in props.collections"
            :key="collection.id"
            :value="collection"
            v-slot="{ active, selected }"
          >
            <li
              :class="[
                active ? 'bg-blue-100 text-blue-900 dark:bg-primary dark:text-white' : 'text-gray-900',
                'relative cursor-pointer select-none py-2 pl-10 pr-4',
              ]"
            >
              <span
                :class="[selected ? 'font-medium' : 'font-normal',
                  'block truncate dark:text-white',
                ]"
              >
                {{ collection.name }}
              </span>
              <span
                v-if="selected"
                class="absolute inset-y-0 left-0 flex items-center pl-3 text-primary dark:text-white"
              >
                <CheckIcon class="h-5 w-5" aria-hidden="true" />
              </span>
            </li>
          </ListboxOption>
          <div class="mb-2 mx-2">
            <c-dropdown-divider />
            <div :class="['hover:bg-gray-100 hover:dark:bg-gray-800 hover:outline-none', 'block px-3 py-1 text-sm/6 text-gray-900 rounded-lg dark:text-gray-400 cursor-pointer']">
              + Ajouter un compte de facturation
            </div>
          </div>
        </ListboxOptions>
      </transition>
    </div>
  </Listbox>
</template>

<script setup lang="ts">
import {
  Listbox,
  ListboxButton,
  ListboxOptions,
  ListboxOption,
} from '@headlessui/vue'
import { CheckIcon } from '@heroicons/vue/20/solid'
import CDropdownDivider from "~/components/dropdown/CDropdownDivider.vue";
import CDropdownItem from "~/components/dropdown/CDropdownItem.vue";

interface Props {
  collections: Array<any>;
  modelValue: { id?: string; name: string };
  // placeholder: string;
}

const props = defineProps<Props>()
const emit = defineEmits(['update:modelValue'])
const selectedItem = ref<{ name: string, id?: string }>()

watch(() => props.modelValue, () => {
  selectedItem.value = props.modelValue
}, { immediate: true })

watch(selectedItem, () => {
  emit('update:modelValue', selectedItem.value)
})
</script>

<style scoped>

</style>