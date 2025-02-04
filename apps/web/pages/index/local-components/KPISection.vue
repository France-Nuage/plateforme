<template>
  <div class="bg-white py-8 sm:py-8">
    <div class="mx-auto max-w-7xl px-6 lg:px-8">
      <div class="mx-auto mt-16 max-w-2xl sm:mt-20 lg:mt-24 lg:max-w-none">
        <div
          v-for="feature in features"
          :key="feature.name"
          :class="[
            'md:flex items-center gap-24',
            {
              'md:flex-row-reverse': feature.reverse,
            },
          ]"
        >
          <div>
            <component :is="feature.illustration" />
          </div>
          <div>
            <h3 class="text-5xl flex-auto mb-4">{{ feature.title }}</h3>
            <p class="text-md text-gray-600 flex-auto">
              {{ feature.description }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import OpenSourceIllustration from "~/pages/index/local-components/fragments/OpenSourceIllustration.vue";
import ControlDestinyIllustration from "./fragments/ControlDestinyIllustration.vue";
import ControlHostingIllustration from "./fragments/ControlHostingIllustration.vue";

const features = [
  {
    title: "Gardez le contrôle de votre destin.",
    description:
      "Reprenez votre autonomie vis-à-vis de votre destin. Avec France-Nuage, simplifiez la maintenance de votre infrastructure tout en gardant le contrôle sur son hébergement.",
    href: "#",
    illustration: ControlDestinyIllustration,
    reverse: false,
  },
  {
    title: "Adoptez l’open-source",
    description:
      "Fini les évolutions de licences de vos logiciels propriétaires en boites noires. Fini les dépenses inutiles monstrueuses à chaque nouveau besoin. Fini le vendor lock-in, bonjour la réversibilité totale, le rêve de chaque DSI et CTO, vous êtes libres de récupérer vos applications et données à tout moment, en un clique.",
    href: "#",
    illustration: OpenSourceIllustration,
    reverse: true,
  },
  {
    title: "Gardez la main sur votre hébergement",
    description:
      "France-Nuage vous offre la flexibilité de gérer la maintenance de vos propres serveurs. Vous ne possédez pas de serveur ? Aucun problème, vous pouvez aussi opter pour un hébergement France-Nuage pour un prix concurrentiel. Vous pourrez migrer vos applications et données rapidement vers vos propres serveurs plus tard.",
    href: "#",
    illustration: ControlHostingIllustration,
    reverse: false,
  },
];
</script>
