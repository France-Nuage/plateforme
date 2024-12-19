<template>
  <c-drawer v-model="isOpen" title="plop">
    <template #header>
      <c-drawer-header
          title="Modifier l'accès à l'organization"
      />
    </template>

    <div class="flex flex-col gap-4">

      <div class="relative flex items-center space-x-6 rounded-lg border px-6 py-5 shadow-sm border-gray-300 bg-white hover:border-gray-400 dark:bg-gray-800 dark:border-gray-700">
        <div class="shrink-0">
          <c-avatar size="xl" />
        </div>
        <div class="min-w-0 flex-1">
          <a href="#" class="focus:outline-none">
            <span class="absolute inset-0" aria-hidden="true" />
            <p class="font-medium text-gray-900 dark:text-gray-200">Alexandre Cailler</p>
            <p class="truncate text-sm text-gray-500 dark:text-gray-300">alex02.cailler@gmail.com</p>
          </a>
        </div>

        <!--          <div>-->
        <!--            <span class="text-sm text-gray-400">-->
        <!--              voir le profile-->
        <!--            </span>-->
        <!--          </div>-->
      </div>

      <c-alert variant="information" title="A noter">
        <div class="mb-3">
          Les rôles sont composés d'ensemble d'autorisations et déterminent ce que le compte peut faire avec cette resource.
        </div>

        <c-button size="sm">En savoir plus</c-button>
      </c-alert>

      <ul role="list" class="divide-y divide-gray-100 dark:divide-gray-800">
        <li
            v-for="i in 5"
            class="flex items-center justify-between gap-x-6 py-5"
        >
          <div class="min-w-0 w-full">
            <div class="flex justify-between items-start gap-x-3 w-full">
              <div class="flex-1 flex items-center gap-x-3">
                <icon name="solar:key-bold-duotone" size="20px" class="text-gray-300" />
                <p class="text-sm/6 font-normal text-gray-900 dark:text-gray-400">roles/resourcemanager.organizationAdmin</p>
              </div>
              <div class="flex gap-1">
                <nuxt-link to="" class="text-primary text-xs">Voir le rôle</nuxt-link>
              </div>
            </div>
          </div>
        </li>
      </ul>

      <c-modal v-model="modalIsOpen" title="plop">
        <c-text-field id="" name="" type="" model-value="" placeholder="Rechercher un rôle" />
      </c-modal>

    </div>
    <template #footer>
      <div class="px-4 pb-6">
        <c-button block @click="modalIsOpen = true">Ajouter un rôle</c-button>
      </div>
    </template>
  </c-drawer>
</template>

<script setup lang="ts">
import CTextField from "~/components/forms/CTextField.vue";
import CAlert from "~/components/alert/CAlert.vue";
import CDrawer from "~/components/drawer/CDrawer.vue";
import CModal from "~/components/modal/CModal.vue";
import CDrawerHeader from "~/components/drawer/CDrawerHeader.vue";
import CAvatar from "~/components/avatar/CAvatar.vue";
import CButton from "~/components/forms/CButton.vue";

const { organization } = storeToRefs(useNavigationStore())
const modalIsOpen = ref(false)
const isOpen = ref(false)
</script>


<style scoped>

</style>