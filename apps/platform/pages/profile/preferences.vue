<template>
  <nuxt-layout name="settings">
    <div class="w-6/12 flex flex-col gap-8">
      <c-card>
        <c-card-header title="Information du profile" />
        <c-card-body>
          <div class="grid grid-cols-12 gap-4">

            <div class="col-span-3">
              <c-label label="Nom" for="lastname" />
            </div>
            <div class="col-span-9">
              <c-text-field id="lastname" required name="lastname" type="text" v-model="formData.lastname" />
            </div>

            <div class="col-span-3">
              <c-label label="Prénom" for="firstname" />
            </div>
            <div class="col-span-9">
              <c-text-field id="firstname" required name="firstname" type="text" v-model="formData.firstname" />
            </div>

          </div>
        </c-card-body>
        <c-card-footer>
          <div class="flex justify-end gap-4 w-full">
            <c-button variant="filled" size="sm" @click="onCancel">Annuler</c-button>
            <c-button variant="success" size="sm" @click="onSubmit" :loading="loading">Valider</c-button>
          </div>
        </c-card-footer>
      </c-card>

      <c-card>
        <c-card-header title="Apparence" />
        <c-card-body>
          <div class="grid grid-cols-12 gap-4">
            <div class="col-span-3">
              <c-label label="Nom" for="lastname" />
            </div>
            <div class="col-span-9">
              <p class="text-sm dark:text-gray-400">France Nuage will use your selected theme</p>
            </div>

            <div class="col-span-3">
              <c-label light label="Choose how Supabase looks to you. Select a single theme, or sync with your system." for="lastname" />
            </div>
            <div class="col-span-9">
              <c-theme-selector />
            </div>
          </div>
        </c-card-body>
      </c-card>

      <c-card>
        <c-card-header title="Zone de danger"  />
        <c-card-body>
          <c-alert title="Request for account deletion" variant="danger">
            <div class="mb-3">
              Deleting your account is permanent and cannot be undone. Your data will be deleted within 30 days, except we may retain some metadata and logs for longer where required or permitted by law.
            </div>

            <c-button variant="danger" size="sm">Request to delete account</c-button>
          </c-alert>
        </c-card-body>
      </c-card>

    </div>
  </nuxt-layout>
</template>

<script setup lang="ts">
import CCard from "~/components/card/CCard.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CCardFooter from "~/components/card/CCardFooter.vue";
import CTextField from "~/components/forms/CTextField.vue";
import CLabel from "~/components/forms/CLabel.vue";
import CButton from "~/components/forms/CButton.vue";
import CCardHeader from "~/components/card/CCardHeader.vue";
import CThemeSelector from "~/components/CThemeSelector.vue";
import CAlert from "~/components/alert/CAlert.vue";

const { update } = useAuthStore()
const { me } = storeToRefs(useAuthStore())
const formData = ref({
  lastname: '',
  firstname: '',
})
const loading = ref(false)

onMounted(() => {
  if (me.value) {
    formData.value.lastname = me.value.lastname;
    formData.value.firstname = me.value.firstname;
  }
})

watch(() => me.value, (newValue) => {
  formData.value.lastname = newValue.lastname;
  formData.value.firstname = newValue.firstname;
})

const onCancel = () => {
  formData.value.lastname = me.value.lastname;
  formData.value.firstname = me.value.firstname;
}

const onSubmit = () => {
  loading.value = true;
  update(formData.value).finally(() => { loading.value = false; })
}
</script>
