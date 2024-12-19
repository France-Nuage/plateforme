<template>
  <nuxt-layout>
     <component
       v-if="resource?.type"
       :is="getCurrentComponent"
     />
  </nuxt-layout>
</template>

<script setup lang="ts">
import SettingOrganization from './local-components/setting-organization.vue'
import SettingFolder from './local-components/setting-folder.vue'
import SettingProject from './local-components/setting-project.vue'

const { resource } = storeToRefs(useNavigationStore())
const getCurrentComponent = computed(() => {
  const _ = {
    'organization': SettingOrganization,
    'folder': SettingFolder,
    'project': SettingProject
  }

  return resource.value?.type && _[resource.value?.type]
})
</script>
