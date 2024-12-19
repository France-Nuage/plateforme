<template>
  <Dialog
      class="fixed inset-0"
      style="z-index: 51"
      :open="props.modelValue"
      @close="$emit('update:modelValue', $event)"
  >
    <DialogBackdrop
        transition
        class="fixed inset-0 z-50 bg-zinc-400/25 backdrop-blur-sm data-[closed]:opacity-0 data-[enter]:duration-300 data-[leave]:duration-200 data-[enter]:ease-out data-[leave]:ease-in dark:bg-black/40"
    />
    <div class="fixed inset-0 overflow-y-auto px-4 py-4 sm:px-6 sm:py-20 md:py-32 lg:px-8 lg:py-[15vh]">
      <DialogPanel
          transition
          class="mx-auto transform-gpu overflow-hidden rounded-lg bg-zinc-50 shadow-xl ring-1 ring-zinc-900/7.5 data-[closed]:scale-95 data-[closed]:opacity-0 data-[enter]:duration-300 data-[leave]:duration-200 data-[enter]:ease-out data-[leave]:ease-in sm:max-w-xl dark:bg-zinc-900 dark:ring-zinc-800"
      >
        <c-modal-header v-if="!props.noHeader || props.title" :title="props.title!" :description="props.description" />
        <slot />
        <c-modal-footer v-if="!noFooter" @add="onAdd" @cancel="onCancel" :loading="props.loading"/>
      </DialogPanel>
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import {Dialog, DialogBackdrop, DialogPanel} from "@headlessui/vue";
import CModalHeader from "~/components/modal/CModalHeader.vue";
import CModalFooter from "~/components/modal/CModalFooter.vue";

interface Props {
  modelValue?: boolean;
  noHeader?: boolean;
  noFooter?: boolean;
  title?: string;
  description?: string;
  loading?: boolean;
}

const props = defineProps<Props>()
const emit = defineEmits(['cancel', 'add', 'update:modelValue'])

const onAdd = (event) => {
  emit('add', event)
}
const onCancel = (event) => {
  emit('cancel', event)
  emit('update:modelValue', false)
}
</script>
