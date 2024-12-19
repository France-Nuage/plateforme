<template>
  <div class="flex flex-col gap-4">

    <div>
      <span class="font-semibold dark:text-gray-300">Informations</span>
    </div>

    <c-text-field
      label="Nom"
      placeholder="Nom du compte de facturation"
      id="name"
      name="name"
      type="text"
      v-model="formData.name"
    />
    <c-text-field
      id="email"
      name="email"
      type="text"
      label="Adresse E-mail"
      placeholder="john@france-nuage.fr"
      description="All billing correspondence will go to this email"
      v-model="formData.emailRecipient"
    />

    <div>
      <span class="font-semibold dark:text-gray-300">Billing Address</span>
    </div>

    <c-text-field
        id="name"
        name="name"
        type="text"
        placeholder="Address line 1"
        v-model="formData.emailRecipient"
    />
    <c-text-field
        id="slug"
        name="slug"
        type="text"
        placeholder="Address line 2"
        v-model="formData.emailRecipient"
    />

    <div class="flex gap-4 w-full">
      <c-text-field
          id="country"
          name="country"
          type="text"
          class="w-full"
          placeholder="Country"
          v-model="formData.emailRecipient"
      />
      <c-text-field
          id="postal_code"
          name="postal_code"
          type="text"
          class="w-full"
          placeholder="Code postal"
          v-model="formData.emailRecipient"
      />
    </div>

  </div>
</template>

<script setup lang="ts">
import CTextField from "~/components/forms/CTextField.vue";

const formData = ref({ name: '', emailRecipient: '' })
</script>

<style scoped>

</style>