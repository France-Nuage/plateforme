<template>
  <div>
    <c-label :label="props.label" for="" />
    <div class="flex gap-16">

      <div class="flex gap-4 w-full">
        <div class="dark:text-gray-400">{{ props.min }}</div>
        <div class="mt-4 w-full">
          <input type="range" :min="props.min" :max="props.max" :step="props.step" class="w-full" v-model="value" />
        </div>
        <div class="dark:text-gray-400">{{ props.max }}</div>
      </div>

      <div class="flex gap-4 justify-between items-center border rounded-lg px-4 dark:border-gray-500" style="width: 150px; max-width: 150px">
        <input
          type="number"
          class="bg-transparent border-none p-0 focus:border-none focus:ring-0 focus:outline-none appearance-none"
          :min="props.min"
          :max="props.max"
          :step="props.step"
          v-model="value"
          style="-moz-appearance: textfield; -webkit-appearance: none; margin: 0;"
        />
        <div class="dark:text-gray-400">{{ props.unit }}</div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import CLabel from "~/components/forms/CLabel.vue";

interface Props {
  min: number | string;
  max: number | string;
  label: string;
  unit: string;
  modelValue: number | string;
  step: number | string;
}

const props = withDefaults(defineProps<Props>(), {
  min: 1,
  max: 80,
  step: 1,
})

const value = ref(0)
</script>