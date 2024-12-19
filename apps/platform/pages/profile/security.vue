<template>
  <nuxt-layout name="settings">
    <div class="w-6/12 flex flex-col gap-8">
      <c-card>
        <c-card-header title="Rénitialisation de mot de passe" />
        <c-card-body>
          <div class="grid grid-cols-12 gap-4">

            <div class="col-span-3">
              <c-label label="Nouveau mot de passe" for="password" />
            </div>
            <div class="col-span-9">
              <c-text-field id="password" required name="password" type="password" v-model="formData.newPassword" />
            </div>

            <div class="col-span-3">
              <c-label label="Mot de passe actuel" for="currentPassword" description="Pour modifier votre mot de passe, nous avons besoin de votre mot de passe actuel." />
            </div>
            <div class="col-span-9">
              <c-text-field id="currentPassword" required name="currentPassword" type="password" v-model="formData.currentPassword" />
            </div>

          </div>
        </c-card-body>
        <c-card-footer>
          <div class="flex justify-end gap-4 w-full">
            <c-button variant="success" size="sm" @click="onSubmit" :loading="loading">Valider</c-button>
          </div>
        </c-card-footer>
      </c-card>
    </div>
  </nuxt-layout>
</template>

<script setup lang="ts">
import CTextField from "~/components/forms/CTextField.vue";
import CLabel from "~/components/forms/CLabel.vue";
import CCard from "~/components/card/CCard.vue";
import CCardHeader from "~/components/card/CCardHeader.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CButton from "~/components/forms/CButton.vue";
import CCardFooter from "~/components/card/CCardFooter.vue";

const formData = ref({
  newPassword: '',
  currentPassword: '',
})

const { } = useAuthStore()
const loading = ref(false)

const onSubmit = async () => {
  loading.value = true
}
</script>
