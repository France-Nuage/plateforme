<template>
  <c-card class="w-11/12 md:w-8/12 lg:w-5/12 mx-auto mt-24">
    <c-card-header title="Associer un nouveau compte de paiement" />
    <c-card-body>

      <div class="grid grid-cols-2 gap-8">
        <payment-method variant="visa" />
        <payment-method variant="apple" />
        <payment-method variant="mastercard" />
        <payment-method-add @click="modalIsOpen = true" />
      </div>

    </c-card-body>

    <c-card-footer>
      <div>
        <c-button size="sm" variant="filled" @click="router.go(-1)">Retour</c-button>
      </div>
      <div>
        <c-button size="sm">Valider</c-button>
      </div>
    </c-card-footer>
  </c-card>

  <c-modal v-model="modalIsOpen" title="Ajouter un moyen de paiement">
    <c-modal-body>
      <payment-method-form />
    </c-modal-body>
  </c-modal>
</template>

<script setup lang="ts">
import CCard from "~/components/card/CCard.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CCardHeader from "~/components/card/CCardHeader.vue";
import PaymentMethodAdd from "~/components/payment/PaymentMethodAdd.vue";
import CCardFooter from "~/components/card/CCardFooter.vue";
import CButton from "~/components/forms/CButton.vue";
import CModal from "~/components/modal/CModal.vue";
import CModalBody from "~/components/modal/CModalBody.vue";

const router = useRouter()
const modalIsOpen = ref(false)

</script>
