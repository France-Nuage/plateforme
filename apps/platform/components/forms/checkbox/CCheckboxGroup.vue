<template>
  <div class="flex flex-col">
    <slot></slot>
  </div>
</template>

<script>
import { provide, ref } from "vue";

export default {
  props: {
    modelValue: Array,
  },
  setup(props, { emit }) {
    const selected = ref(props.modelValue || []);

    const updateValue = (value) => {
      const index = selected.value.indexOf(value);
      if (index === -1) {
        selected.value.push(value);
      } else {
        selected.value.splice(index, 1);
      }
      emit("update:modelValue", selected.value);
    };

    provide("checkboxGroup", { selected, updateValue });

    return { selected };
  },
};
</script>
