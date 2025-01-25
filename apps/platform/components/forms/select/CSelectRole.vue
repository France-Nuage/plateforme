<template>
  <Listbox v-model="selectedItem">
    <div class="relative mt-1 w-full">
      <ListboxButton class="focus:border-none focus:outline-none w-full">
        <c-text-field
            :id="props.id"
            :name="props.name"
            type="text"
            :placeholder="props.placeholder"
            v-model="textFieldValue"
            readonly
            class="cursor-pointer"
        />
      </ListboxButton>
      <transition
          leave-active-class="transition duration-100 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
      >
        <ListboxOptions
            class="absolute mt-1 max-h-60 w-full overflow-auto rounded-md bg-white py-1 text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm"
        >

          <div class="flex items-center bg-gray-100 px-4 py-2 -mt-1">

            <icon name="solar:filter-bold" size="20px" class="text-gray-400" />

            <input
                type="text"
                placeholder="Rechercher un role ou une autorisation"
                class="focus:outline-none focus:ring-0 border-0 w-full py-2 px-3 rounded-md bg-transparent"
                v-model="searchTerms"
            />

          </div>

          <ListboxOption
              v-for="role in rolesFiltered"
              :key="role.id"
              :value="role"
              v-slot="{ active, selected }"
          >
            <li
                :class="[
                active ? 'bg-blue-100 text-blue-900' : 'text-gray-900',
                'relative cursor-default select-none py-2 pl-10 pr-4',
              ]"
            >
              <span
                  :class="[selected ? 'font-medium' : 'font-normal',
                  'block truncate',
                ]"
              >
                {{ role.id }}
              </span>
              <span
                  v-if="selected"
                  class="absolute inset-y-0 left-0 flex items-center pl-3 text-primary"
              >
                <CheckIcon class="h-5 w-5" aria-hidden="true" />
              </span>
            </li>
          </ListboxOption>
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
import CTextField from "~/components/forms/CTextField.vue";

interface Props {
  modelValue: any;
  placeholder?: string;
  name: string;
  id: string;
  rolesSelected: Array<any>
}

const props = defineProps<Props>()
const emit = defineEmits(['update:modelValue'])
const selectedItem = ref(null)
const textFieldValue = ref('')

watch(() => props.modelValue, () => {
  selectedItem.value = props.modelValue
})

watch(selectedItem, () => {
  textFieldValue.value = selectedItem.value.name
  emit('update:modelValue', selectedItem.value)
})

const { roles } = storeToRefs(useRoleStore())
const { loadRoles } = useRoleStore()
onMounted(() => {
  loadRoles()
})

const searchTerms = ref('')

const rolesFiltered = computed(() => {
  const filteredValue = searchTerms.value.trim() ? roles.value.filter((role) => {
    return role.id.toLowerCase().includes(searchTerms.value.toLowerCase())
  }) : roles.value
  return filteredValue.filter((item) => !props.rolesSelected.map(i => i.id).includes(item.id))
})
</script>
