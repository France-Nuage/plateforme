<template>
  <footer class="bg-gray-900">
    <div class="mx-auto max-w-7xl px-6 py-16 sm:py-24 lg:px-8 lg:py-32">
      <!-- Mise en page en grille pour gérer différentes tailles d'écran -->
      <div class="grid gap-8 xl:grid-cols-3 xl:gap-8">
        <!-- Logo, France Nuage et informations de contact -->
        <div class="flex flex-col space-y-4 col-span-3 xl:col-span-1">
          <div class="flex items-center space-x-4">
            <Logo />
            <span class="font-semibold text-white">France Nuage</span>
          </div>
          <div class="text-sm text-gray-400">
            <p>
              Email :
              <a href="mailto:bonjour@france-nuage.fr" class="hover:text-white"
                >bonjour@france-nuage.fr</a
              >
            </p>
            <p>
              Téléphone :
              <a href="tel:+33781479170" class="hover:text-white"
                >+33 7 81 47 91 70</a
              >
            </p>
          </div>
        </div>

        <!-- Colonnes de navigation à droite pour les grands écrans, sous le logo pour les petits écrans (2/3) -->
        <div
          class="col-span-3 xl:col-span-2 grid grid-cols-2 gap-8 xl:grid-cols-4 mt-8 xl:mt-0"
        >
          <div>
            <h3 class="text-sm/6 font-semibold text-white">Solutions</h3>
            <ul role="list" class="mt-6 space-y-4">
              <li v-for="item in navigation.solutions" :key="item.name">
                <a
                  :href="item.href"
                  class="text-sm/6 text-gray-400 hover:text-white"
                  >{{ item.nameFr }}</a
                >
              </li>
            </ul>
          </div>
          <div>
            <h3 class="text-sm/6 font-semibold text-white">Support</h3>
            <ul role="list" class="mt-6 space-y-4">
              <li v-for="item in navigation.support" :key="item.name">
                <a
                  :href="item.href"
                  class="text-sm/6 text-gray-400 hover:text-white"
                  >{{ item.nameFr }}</a
                >
              </li>
            </ul>
          </div>
          <div>
            <h3 class="text-sm/6 font-semibold text-white">Entreprise</h3>
            <ul role="list" class="mt-6 space-y-4">
              <li v-for="item in navigation.company" :key="item.name">
                <a
                  :href="item.href"
                  class="text-sm/6 text-gray-400 hover:text-white"
                  >{{ item.nameFr }}</a
                >
              </li>
            </ul>
          </div>
          <div>
            <h3 class="text-sm/6 font-semibold text-white">Légal</h3>
            <ul role="list" class="mt-6 space-y-4">
              <li v-for="item in navigation.legal" :key="item.name">
                <a
                  :href="item.href"
                  class="text-sm/6 text-gray-400 hover:text-white"
                  >{{ item.nameFr }}</a
                >
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </footer>
</template>

<script setup>
import Logo from "~/components/Logo.vue";
const navigation = {
  solutions: [
    { name: "Marketing", nameFr: "Marketing", href: "/marketing" },
    { name: "Automation", nameFr: "Automatisation", href: "/automation" },
    { name: "Insights", nameFr: "Perspectives", href: "/insights" },
  ],
  support: [
    {
      name: "Submit ticket",
      nameFr: "Soumettre un ticket",
      href: "/submitTicket",
    },
    { name: "Documentation", nameFr: "Documentation", href: "/documentation" },
    { name: "Guides", nameFr: "Guides", href: "/guides" },
  ],
  company: [{ name: "About", nameFr: "À propos", href: "/about" }],
  legal: [
    { name: "Notice", nameFr: "Mentions légales", href: "/legalNotice" },
    {
      name: "Terms of Service",
      nameFr: "Conditions d'utilisation",
      href: "/termsOfService",
    },
    {
      name: "Privacy Policy",
      nameFr: "Politique de confidentialité",
      href: "/privacyPolicy",
    },
    {
      name: "License",
      nameFr: "Licence",
      href: "https://gitlab.com/getnobullshit/france-nuage/website/-/raw/master/LICENCE?ref_type=heads",
    },
  ],
};
</script>
