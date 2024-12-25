<template>
  <div class="flex flex-col gap-8">
    <c-card>
      <c-card-body>

        <div class="grid grid-cols-12">

          <div class="col-span-3">
            <c-label label="Paramètres général" for="#" />
          </div>
          <div class="col-span-9 flex flex-col gap-4">
            <c-text-field
              id="name"
              name="name"
              type="text"
              label="Project name"
              v-model="formData.name"
            />
            <c-text-field
              id="slug"
              name="slug"
              type="text"
              label="Slug"
              v-model="project.id"
              disabled
            />
          </div>

        </div>

      </c-card-body>
      <c-card-footer>
        <div class="flex w-full justify-end gap-4">
          <c-button variant="filled" size="sm" @click="onCancel">Annuler</c-button>
          <c-button variant="success" size="sm" @click="onSubmit" :loading="loading">Enregistrer</c-button>
        </div>
      </c-card-footer>
    </c-card>

    <c-card>
      <c-card-header title="Zone de danger"  />
      <c-card-body>
        <c-alert title="Request for project deletion" variant="danger">
          <div class="mb-3">
            Deleting your project is permanent and cannot be undone. Your data will be deleted within 30 days, except we may retain some metadata and logs for longer where required or permitted by law.
          </div>

          <c-button variant="danger" size="sm">Request to delete instance</c-button>
        </c-alert>
      </c-card-body>
    </c-card>
  </div>
</template>

<script setup lang="ts">
import CCard from "~/components/card/CCard.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CCardFooter from "~/components/card/CCardFooter.vue";
import CButton from "~/components/forms/CButton.vue";
import CLabel from "~/components/forms/CLabel.vue";
import CTextField from "~/components/forms/CTextField.vue";
import CCardHeader from "~/components/card/CCardHeader.vue";
import CAlert from "~/components/alert/CAlert.vue";

const { project } = storeToRefs(useNavigationStore())
const loading = ref(false)
const formData = ref({
  name: '',
})

onMounted(() => {
  if (project.value) {
    formData.value.name = project.value.name;
  }
})

watch(() => project.value, (newValue) => {
  formData.value.name = newValue.name;
})

const onCancel = () => {
  formData.value.name = project.value.name;
}

const onSubmit = () => {
  loading.value = true;
}
</script>
