<template>
  <div class="bg-gray-50 text-gray-900 flex items-center justify-center mt-32">
    <div class="container mx-auto flex flex-col lg:flex-row items-center justify-center px-6 py-12">
      <div class="text-center lg:text-left max-w-sm">
        <!-- Stepper -->
        <h2 class="text-2xl font-semibold mb-2 dark:text-gray-200">Créer une instance de Machine Virtuelle</h2>
        <!-- Texte explicatif -->
        <p class="text-gray-600 mb-6 dark:text-gray-400">Nous allons suivre vos étapes de configuration et vous aider à créer rapidement votre nouvel item.</p>

        <ol class="space-y-4 mb-8">
          <li class="flex items-center justify-center lg:justify-start">
            <div class="w-8 h-8 flex items-center justify-center rounded-full border-2 border-primary text-white mr-4 text-sm">1</div>
            <span class="text-gray-700 dark:text-gray-400">Choisissez une localisation</span>
          </li>
          <li class="flex items-center justify-center lg:justify-start">
            <div class="w-8 h-8 flex items-center justify-center rounded-full border-2 border-primary text-white mr-4 text-sm">2</div>
            <span class="text-gray-700 dark:text-gray-400">Sélectionner les performances</span>
          </li>
          <li class="flex items-center justify-center lg:justify-start">
            <div class="w-8 h-8 flex items-center justify-center rounded-full border-2 border-primary text-white mr-4 text-sm">3</div>
            <span class="text-gray-700 dark:text-gray-400">Choisir le bootage</span>
          </li>
        </ol>

        <!-- CTA -->
       <c-button size="sm" variant="success" @click="router.push('/instances/create')">Créer une Machine Virtuelle</c-button>

        <div class="flex flex-col space-y-2 items-center lg:items-start mt-4">
          <c-sidebar-navigation-item title="Guides" icon="square-bottom-up-line-duotone" to="/instances" />
          <c-sidebar-navigation-item title="Api référence" icon="square-bottom-up-line-duotone" to="/services" />
        </div>
      </div>
      <div class="mb-8 lg:mb-0 lg:mr-16 max-w-md flex justify-center">
        <img src="https://uptime.betterstack.com/assets/blankslates/monitors-light-bf8b2157bcce1301750c14dcf564d35c64b1da2c841382a5f37a2504d4f096ef.png" />
      </div>
    </div>
  </div>
</template>

<script setup>
import CButton from "~/components/forms/CButton.vue";
import CSidebarNavigationItem from "~/components/sidebar/CSidebarNavigationItem.vue";

const router = useRouter();
</script>