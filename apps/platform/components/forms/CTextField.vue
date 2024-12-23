<template>
  <div>
    <c-label
      v-if="props.label"
      :for="props.id || props.name"
      :label="props.label"
      class="block text-sm/6 font-medium"
    />
    <div class="mt-2">
      <input
        :id="props.id || props.name"
        :name="props.name"
        :type="props.type"
        :autocomplete="props.autocomplete"
        :required="props.required"
        :placeholder="props.placeholder"
        :readonly="props.readonly"
        :class="['block w-full rounded-md border-0 py-1.5 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-primary sm:text-sm/6 dark:bg-gray-700 dark:ring-gray-500 dark:text-gray-200', {
          'dark:text-gray-400 cursor-not-allowed': props.disabled
        }]"
        v-bind="$attrs"
        :value="props.modelValue"
        @input="onInput"
        :disabled="props.disabled"
      />
    </div>
    <div v-if="props.description" class="text-gray-500 mt-2 text-sm">{{ props.description }}</div>
  </div>
</template>

<script setup lang="ts">
import CLabel from "~/components/forms/CLabel.vue";

interface Props {
  required?: boolean;
  autocomplete?: string;
  name: string;
  id: string;
  type: string;
  placeholder?: string;
  label?: string;
  description?: string;
  modelValue: string;
  readonly?: boolean;
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), { type: 'text' });
const emit = defineEmits(['update:modelValue'])

const onInput = (event: InputEvent) => {
  emit('update:modelValue', (event.target as HTMLInputElement).value);
}
</script>

<style scoped>

</style>