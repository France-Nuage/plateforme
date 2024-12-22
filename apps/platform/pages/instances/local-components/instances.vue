<template>
  <c-card>
    <c-card-header
      title="Configuration de la machine"
      description="Types de machines pour les charges de travail courantes permettant d'optimiser les coûts et la flexibilité."
    />
    <c-card-body>

      <div class="flex flex-col gap-8">
        <c-input-range min="2" max="80" unit="vCPU" :step="1" v-model="heartNumber" label="Coeurs" />
        <c-input-range min="2" max="80" unit="Go" :step="1" v-model="ramNumber" label="Mémoire" />
      </div>
      
    </c-card-body>
  </c-card>
</template>

<script setup lang="ts">
import CCardHeader from "~/components/card/CCardHeader.vue";
import CCard from "~/components/card/CCard.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CInstancePriceLine from "~/components/instances/CInstancePriceLine.vue";
import { RadioGroup, RadioGroupOption } from '@headlessui/vue'
import { ref } from "vue";
import CAction from "~/components/pannel/CAction.vue";
import CLabel from "~/components/forms/CLabel.vue";
import CInputRange from "~/components/forms/CInputRange.vue";

const heartNumber = ref(2);
const ramNumber = ref(2);
</script>

<style scoped>

</style>