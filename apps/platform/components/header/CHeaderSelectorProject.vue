<template>
  <c-header-selector
    v-if="folder"
    :items="projects.filter((_) => _.folderId === folder.id).map((_) => ({ value: _.id, name: _.name }))"
    type="project"
    :selected="selected"
    @select="onChangeProject"
    @add="modalIsOpen = true"
  />
  <c-modal v-model="modalIsOpen" title="Ajouter un projet" @add="onSubmit" :loading="loading">
    <c-modal-body>
      <c-text-field
        id="name"
        name="name"
        type="text"
        label="Nom du projet"
        v-model="formData.name"
      />
    </c-modal-body>
  </c-modal>
</template>

<script setup lang="ts">
import CHeaderSelector from "~/components/header/CHeaderSelector.vue";
import CTextField from "~/components/forms/CTextField.vue";
import CModal from "~/components/modal/CModal.vue";
import CModalBody from "~/components/modal/CModalBody.vue";

const { createResource } = useNavigationStore()
const { projects, folder, project } = storeToRefs(useNavigationStore())
const router = useRouter()
const route = useRoute()
const modalIsOpen = ref(false)
const formData= ref({ name: '' })
const loading = ref(false)
const selected = computed(() => {
  return project.value ? {
    name: project.value.name,
    value: project.value.id
  } : null
})

const onChangeProject = (id) => {
  router.push({ ...route, query: {
      organization: route.query.organization,
      folder: route.query.folder,
      project: id,
    }
  });
}

const onSubmit = () => {
  loading.value = true
  createResource({...formData.value, folderId: route.query.folder}, { type: 'project' })
      .then((response) => {
        router.push({ ...route, query: { ...route.query, project: response.id }})
        modalIsOpen.value = false
      })
      .finally(() => {
        loading.value = false
      })
}
</script>
