<template>
  <c-header-selector
    :items="folders.filter((_) => _.organizationId === organization.id).map((_) => ({ value: _.id, name: _.name }))"
    :selected="selected"
    type="folder"
    @select="onFolderSelected"
    @add="modalIsOpen = true"
  />
  <c-modal v-model="modalIsOpen" title="Ajouter une filial" @add="onSubmit" :loading="loading">
    <c-modal-body>
      <c-text-field
        id="name"
        name="name"
        type="text"
        label="Nom de la filial"
        v-model="formData.name"
      />
    </c-modal-body>
  </c-modal>
</template>

<script setup lang="ts">
import CHeaderSelector from "~/components/header/CHeaderSelector.vue";
import CModal from "~/components/modal/CModal.vue";
import CModalBody from "~/components/modal/CModalBody.vue";
import CTextField from "~/components/forms/CTextField.vue";

const { folder, folders, organization } = storeToRefs(useNavigationStore())
const { createResource } = useNavigationStore()
const router = useRouter()
const route = useRoute()
const modalIsOpen = ref(false)
const loading = ref(false)
const formData= ref({ name: '' })
const selected = computed(() => {
  return folder.value ? { name: folder.value.name, value: folder.value.id } : null
})

const onFolderSelected = (id: string) => {
  router.push({ ...route, query: {
      organization: route.query.organization,
      folder: id
    }
  });
}

const onSubmit = () => {
  loading.value = true
  createResource({...formData.value, organizationId: route.query.organization}, { type: 'folder' })
    .then(response => {
      router.push({ ...route, query: { organization: route.query.organization, folder: response.id }})
      modalIsOpen.value = false
    })
    .finally(() => {
      loading.value = false
    })
}
</script>
