<template>
  <SwitchGroup>
    <div>
      <div class="flex items-center">
        <SwitchLabel v-if="props.label" class="mr-4">{{
          props.label
        }}</SwitchLabel>
        <Switch
          v-model="enabled"
          :class="enabled ? 'bg-primary' : 'bg-gray-200'"
          class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2"
        >
          <span
            :class="enabled ? 'translate-x-6' : 'translate-x-1'"
            class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
          />
        </Switch>
      </div>
      <div v-if="props.description" class="text-gray-500 mt-2 text-sm">
        {{ props.description }}
      </div>
    </div>
  </SwitchGroup>
</template>

<script setup lang="ts">
import { Switch, SwitchGroup, SwitchLabel } from "@headlessui/vue";

interface Props {
  modelValue: boolean;
  label?: string;
  description?: string;
}

const props = defineProps<Props>();
const enabled = ref(false);
const emit = defineEmits(["update:modelValue"]);

watch(
  () => props.modelValue,
  () => {
    enabled.value = props.modelValue;
  },
  { immediate: true },
);

watch(enabled, () => {
  emit("update:modelValue", enabled.value);
});
</script>
