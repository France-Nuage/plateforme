<template>
  <c-card>
    <card-c-card-header
      title="Entrez un nom pour votre instance"
    >
    </card-c-card-header>
    <c-card-body>
      <div class="grid grid-cols-12 w-full">
        <div class="col-span-3">
          <c-label label="Donner un nom à votre instance" for="instance-name" />
        </div>
        <div class="col-span-9">
          <c-text-field
            id="instance-name"
            required
            name="instance-name"
            type="text"
            v-model="value"
            placeholder="Nom de l'instance"
            description="Le nom de votre Instance ne peut contenir que des caractères alphanumériques, des points et des tirets."
          />
        </div>
      </div>
    </c-card-body>
  </c-card>
</template>

<script setup lang="ts">
import CTextField from "~/components/forms/CTextField.vue";
import CCard from "~/components/card/CCard.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CLabel from "~/components/forms/CLabel.vue";

interface Props {
  modelValue: string;
}

const props = defineProps<Props>()
const value = ref()
</script>
