<template>
  <c-card class="w-11/12 md:w-8/12 lg:w-4/12 mx-auto mt-24">
    <c-card-header title="Créer un nouveau compte de paiement" />
    <c-card-body>

      <p class="text-xs mb-8 dark:text-gray-400">
        <span class="font-semibold">This is your payment account within France-Nuage.</span>
        For example, you can use the name of company department.
      </p>

      <div class="flex flex-col gap-8">
        <div class="grid grid-cols-12 w-full">
          <c-label label="Nom" for="name" class="col-span-3" />
          <c-text-field v-model="formData.name" id="name" required name="name" type="text" class="col-span-9" />
        </div>
      </div>

    </c-card-body>
    <c-card-footer>
      <div>
        <c-button variant="filled" size="sm" @click="router.go(-1)">Annulé</c-button>
      </div>
      <div class="flex items-center gap-4">
        <small class="text-xs dark:text-gray-400">Vous pouvez renommer le nom <br> du compte de paiement plus tard</small>
        <c-button size="sm" @click="onSubmit" :loading="loading">Créer un compte de paiement</c-button>
      </div>
    </c-card-footer>
  </c-card>
</template>

<script setup lang="ts">

import CCard from "~/components/card/CCard.vue";
import CCardHeader from "~/components/card/CCardHeader.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CCardFooter from "~/components/card/CCardFooter.vue";
import CLabel from "~/components/forms/CLabel.vue";
import CTextField from "~/components/forms/CTextField.vue";
import CButton from "~/components/forms/CButton.vue";
import { useBillingAccountStore } from "~/stores/billing/billingAccount";

const formData = ref({
  name: "",
})

const router = useRouter()
const loading = ref(false)
const { createBillingAccount } = useBillingAccountStore()
const { folder } = useNavigationStore()
const onSubmit = () => {
  createBillingAccount({ ...formData.value, folderId: folder.id })
}
</script>
