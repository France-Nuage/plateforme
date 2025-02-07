<template>
  <div class="bg-gray-50 h-screen dark:bg-gray-900">
    <c-breadcrumb logo :pages="pages" />
    <nuxt-page />
  </div>
</template>

<script lang="ts" setup>
import CBreadcrumb from "~/components/breadcrumb/CBreadcrumb.vue";
const { organization } = storeToRefs(useOrganizationStore());

const pages = computed(() => {
  let organizationLabel = "Créer une organisation";

  if (organization.value) {
    organizationLabel += ` : (${organization.value.name})`;
  }

  return [
    { name: organizationLabel, href: "#", current: true },
    { name: "Créer un compte", href: "#", current: true },
  ];
});
</script>
