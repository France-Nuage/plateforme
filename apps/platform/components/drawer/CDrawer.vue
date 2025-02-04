<template>
  <TransitionRoot as="template" :show="props.modelValue">
    <Dialog class="relative z-50" @close="$emit('update:modelValue', $event)">
      <TransitionChild
        as="template"
        enter="ease-in-out duration-100"
        enter-from="opacity-0"
        enter-to="opacity-100"
        leave="ease-in-out duration-500"
        leave-from="opacity-100"
        leave-to="opacity-0"
      >
        <div class="fixed inset-0 bg-gray-900/90 transition-opacity" />
      </TransitionChild>

      <div class="fixed inset-0 overflow-hidden">
        <div class="absolute inset-0 overflow-hidden">
          <div
            class="pointer-events-none fixed inset-y-0 right-0 flex max-w-full pl-10"
          >
            <TransitionChild
              as="template"
              enter="transform transition ease-in-out duration-500 sm:duration-500"
              enter-from="translate-x-full"
              enter-to="translate-x-0"
              leave="transform transition ease-in-out duration-500 sm:duration-500"
              leave-from="translate-x-0"
              leave-to="translate-x-full"
            >
              <DialogPanel class="pointer-events-auto w-screen max-w-xl m-3">
                <div
                  class="absolute transition-all"
                  style="left: -1rem; top: 3rem"
                >
                  <button
                    type="button"
                    class="relative p-2 focus:outline-none rounded-full bg-white text-gray-400 hover:text-gray-500 dark:bg-gray-600 dark:text-white"
                    @click="$emit('update:modelValue', false)"
                  >
                    <span class="absolute -inset-2.5" />
                    <span class="sr-only">Close panel</span>
                    <XMarkIcon class="size-6" aria-hidden="true" />
                  </button>
                </div>
                <div
                  class="flex h-full flex-col overflow-y-scroll bg-white dark:bg-gray-900 dark:border dark:border-gray-700 rounded-xl shadow-xl"
                >
                  <slot name="header">
                    <div class="px-4 sm:px-6">
                      <div class="flex items-start justify-between">
                        <DialogTitle
                          v-if="props.title"
                          class="text-base font-semibold text-gray-900 dark:text-gray-300"
                          >{{ props.title }}</DialogTitle
                        >
                      </div>
                    </div>
                  </slot>
                  <div class="relative mt-6 flex-1 px-4 sm:px-6">
                    <slot />
                  </div>
                  <slot name="footer" />
                </div>
              </DialogPanel>
            </TransitionChild>
          </div>
        </div>
      </div>
    </Dialog>
  </TransitionRoot>
</template>

<script setup lang="ts">
import {
  Dialog,
  DialogPanel,
  DialogTitle,
  TransitionChild,
  TransitionRoot,
} from "@headlessui/vue";
import { XMarkIcon } from "@heroicons/vue/24/outline";

interface Props {
  modelValue?: boolean;
  title?: string;
}

const props = defineProps<Props>();
</script>

<style scoped></style>
