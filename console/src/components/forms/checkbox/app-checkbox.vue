<template>
  <div @click="handleToggle">
    <input
      type="checkbox"
      :checked="isCheckedInGroup"
      class="hidden"
      hidden="hidden"
      :name="name"
    />
    <div class="d-flex gap-4">
      <app-checkbox-base :checked="isCheckedInGroup" />
      <label v-if="label" class="label">
        <span class="text-white">{{ props.label }}</span>
        <span v-if="props.description" class="lead">{{ props.description }}</span>
      </label>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { inject, computed } from 'vue'
import AppCheckboxBase from './app-checkbox-base.vue'

interface Props<T = string | number> {
  value?: T
  modelValue?: T | T[]
  onUpdate?: (value: T) => void
  label?: string
  name?: string
  description?: string
}

const emit = defineEmits(['update:modelValue'])
const checkboxGroup = inject('checkboxGroup', null)
const props = defineProps<Props>()

const isCheckedInGroup = computed(() => {
  if (checkboxGroup) {
    return checkboxGroup.selected.value.includes(props.value)
  } else {
    if (props.modelValue instanceof Array) {
      return props.modelValue.includes(props.value)
    }
    return props.modelValue
  }
})

const handleToggle = () => {
  if (checkboxGroup) {
    checkboxGroup.updateValue(props.value)
  } else {
    let model = props.modelValue
    if (model instanceof Array) {
      if (isCheckedInGroup.value) {
        model.splice(model.indexOf(props.value), 1)
      } else {
        model.push(props.value)
      }
    } else {
      model = !model
    }

    emit('update:modelValue', model)
  }
}
</script>
