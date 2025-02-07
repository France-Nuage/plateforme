<template>
  <c-card>
    <c-card-header
      title="Configuration de la machine"
      description="Types de machines pour les charges de travail courantes permettant d'optimiser les coûts et la flexibilité."
    />
    <c-card-body>
      <div class="flex flex-col gap-8">
        <c-input-range
          min="2"
          max="80"
          unit="vCPU"
          :step="1"
          v-model="cpu"
          label="Coeurs"
        />
        <c-input-range
          min="2"
          max="80"
          unit="Go"
          :step="1"
          v-model="ram"
          label="Mémoire"
        />
      </div>
    </c-card-body>
  </c-card>
</template>

<script setup lang="ts">
import CCardHeader from "~/components/card/CCardHeader.vue";
import CCard from "~/components/card/CCard.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CInstancePriceLine from "~/components/instances/CInstancePriceLine.vue";
import { RadioGroup, RadioGroupOption } from "@headlessui/vue";
import { ref } from "vue";
import CAction from "~/components/pannel/CAction.vue";
import CLabel from "~/components/forms/CLabel.vue";
import CInputRange from "~/components/forms/CInputRange.vue";

interface Props {
  modelValue: any;
}

const props = defineProps<Props>();
const cpu = ref(2);
const ram = ref(2);

const emit = defineEmits(["update:modelValue"]);

watch(
  () => [cpu.value, ram.value],
  (value) => {
    emit("update:modelValue", {
      ...props.modelValue,
      cpu: parseInt(value[0]),
      ram: parseInt(value[1]),
    });
  },
);
</script>
