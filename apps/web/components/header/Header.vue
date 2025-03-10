<template>
  <header class="absolute inset-x-0 top-0 z-50">
    <nav
      class="flex items-center justify-between p-6 lg:px-8"
      aria-label="Global"
    >
      <div class="flex lg:flex-1">
        <a href="/" class="-m-1.5 p-1.5 flex gap-4 items-center">
          <Logo /><span class="font-semibold">France Nuage</span>
        </a>
      </div>
      <div class="flex lg:hidden">
        <a
          href="javascript:void(0)"
          class="-m-2.5 inline-flex items-center justify-center rounded-md p-2.5 text-gray-700"
          @click.prevent="mobileMenuOpen = true"
        >
          <span class="sr-only">Ouvrir le menu</span>
          <Bars3Icon class="size-6" aria-hidden="true" />
        </a>
      </div>
      <div class="hidden lg:flex lg:gap-x-12">
        <a
          v-for="item in navigation"
          :key="item.name"
          :href="item.href"
          class="text-sm/6 font-semibold text-gray-900"
          >{{ item.name }}</a
        >
      </div>
      <div class="hidden lg:flex lg:flex-1 lg:justify-end">
        <a
          href="https://plateforme.france-nuage.fr"
          class="text-sm/6 font-semibold text-gray-900"
          >Connexion <span aria-hidden="true">&rarr;</span></a
        >
      </div>
    </nav>
    <Dialog
      class="lg:hidden"
      @close="mobileMenuOpen = false"
      :open="mobileMenuOpen"
    >
      <div class="fixed inset-0 z-50" />
      <DialogPanel
        class="fixed inset-y-0 right-0 z-50 w-full overflow-y-auto bg-white px-6 py-6 sm:max-w-sm sm:ring-1 sm:ring-gray-900/10"
      >
        <div class="flex items-center justify-between">
          <a href="/" class="-m-1.5 p-1.5">
            <span class="sr-only">France Nuage</span>
            <img
              class="h-8 w-auto"
              src="/images/logo_france_nuage.svg"
              alt=""
            />
          </a>
          <a
            href="javascript:void(0)"
            class="-m-2.5 rounded-md p-2.5 text-gray-700"
            @click.prevent="mobileMenuOpen = false"
          >
            <span class="sr-only">Fermer</span>
            <XMarkIcon class="size-6" aria-hidden="true" />
          </a>
        </div>
        <div class="mt-6 flow-root">
          <div class="-my-6 divide-y divide-gray-500/10">
            <div class="space-y-2 py-6">
              <a
                v-for="item in navigation"
                :key="item.name"
                :href="item.href"
                class="-mx-3 block rounded-lg px-3 py-2 text-base/7 font-semibold text-gray-900 hover:bg-gray-50"
                >{{ item.name }}</a
              >
            </div>
            <div class="py-6">
              <a
                href="https://plateforme.france-nuage.fr"
                class="-mx-3 block rounded-lg px-3 py-2.5 text-base/7 font-semibold text-gray-900 hover:bg-gray-50"
                >Connexion</a
              >
            </div>
          </div>
        </div>
      </DialogPanel>
    </Dialog>
  </header>
</template>

<script setup>
import { ref } from "vue";
import { Dialog, DialogPanel } from "@headlessui/vue";
import { Bars3Icon, XMarkIcon } from "@heroicons/vue/24/outline";
import Logo from "~/components/Logo.vue";

const navigation = [
  { name: "Accueil", href: "/" },
  { name: "À propos", href: "/about" },
  { name: "Contact", href: "/contact" },
];

const mobileMenuOpen = ref(false);
</script>
