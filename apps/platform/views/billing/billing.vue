<template>

  <div class="w-full">
    <div class="grid grid-cols-4 gap-8">

      <c-card-billing-account billing="" />
      <c-card-billing-account billing="" />
      <c-card-billing-account billing="" />
      <c-card-billing-account billing="" />

    </div>

    <c-modal v-model="modalIsOpen" title="Ajouter un compte de facturation">
      <c-modal-body>
        <billing-add-form />
      </c-modal-body>
    </c-modal>

    <div class="mt-16">
      <c-button @click="modalIsOpen = true">Ajouter un compte de facturation</c-button>
    </div>

  </div>

</template>

<script setup lang="ts">
import CCardBillingAccount from "~/components/card/CCardBillingAccount.vue";
import CButton from "~/components/forms/CButton.vue";
import CModal from "~/components/modal/CModal.vue";
import BillingAddForm from "~/pages/settings/local-components/fragments/billingAddForm.vue";
import CModalBody from "~/components/modal/CModalBody.vue";

const modalIsOpen = ref(false)
</script>

<style scoped>

</style>