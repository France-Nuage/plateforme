<template>
  <div class="bg-white py-24 sm:py-32">
    <div class="mx-auto max-w-7xl px-6 lg:px-8">
      <div class="mx-auto max-w-4xl text-center">
        <h2
          class="text-balance text-4xl font-semibold tracking-tight text-center sm:text-5xl"
        >
          Nos experts supervisent et sécurisent H24, vos applications et votre
          infrastructure.
        </h2>
      </div>
      <div class="mt-16 flex justify-center">
        <fieldset aria-label="Payment frequency">
          <SwitchGroup>
            <Switch
              v-model="ownServer"
              :class="ownServer ? 'bg-primary' : 'bg-gray-200'"
              class="relative inline-flex h-6 w-11 items-center rounded-full"
            >
              <span
                :class="ownServer ? 'translate-x-6' : 'translate-x-1'"
                class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
              />
            </Switch>
            <SwitchLabel class="ml-4"
              ><span class="font-semibold">J'ai mes propres serveurs</span> —
              Connectez vos propres serveurs et machines virtuelles à France
              Nuage afin de déployer en un seul clic vos applications, base de
              données etc.</SwitchLabel
            >
          </SwitchGroup>
        </fieldset>
      </div>
      <div
        class="isolate mx-auto mt-10 grid max-w-md gap-8 lg:mx-0 lg:max-w-none justify-center md:grid-cols-2 sm:grid-cols-1 grid-cols-1"
        :class="ownServer ? 'lg:grid-cols-3 ' : 'lg:grid-cols-2'"
      >
        <div
          v-for="tier in tiers.filter(
            (item) => item.alreadyHaveServers === ownServer,
          )"
          :key="tier.id"
          :class="[
            tier.isStartup
              ? 'ring-2 ring-primary bg-primary'
              : 'ring-1 ring-gray-200',
            'rounded-3xl p-8 xl:p-10',
          ]"
        >
          <div class="flex items-center justify-between gap-x-4">
            <p
              class="rounded-full bg-[#CEE9FE] px-2.5 py-1 text-xs/5 font-semibold text-[#Startup]"
            >
              {{ tier.name }}
            </p>
          </div>
          <p class="mt-6 flex items-baseline gap-x-1">
            <span
              :class="[
                tier.isStartup ? 'text-white' : 'text-gray-900',
                'text-4xl font-semibold tracking-tight',
              ]"
              >{{ tier.price }}</span
            >
          </p>
          <a
            href="/contact"
            :class="[
              tier.isStartup
                ? 'bg-white text-primary'
                : 'bg-primary text-white',
              'block w-full text-center mt-6 px-6 py-3 rounded-lg font-semibold shadow-sm hover:opacity-90 transition',
            ]"
          >
            Commencer dès maintenant
          </a>
          <div class="xl:mt-10">
            <p
              v-if="tier.isStartup && tier.alreadyHaveServers"
              class="text-white lowercase"
            >
              * Le plan autonome plus :
            </p>
            <p v-if="tier.isEntreprise" class="lowercase">
              * Le plan startup plus :
            </p>
            <ul role="list" class="mt-8 space-y-3 text-sm/6 text-gray-600">
              <li
                v-for="feature in tier.features"
                :key="feature"
                class="flex gap-x-3"
              >
                <CheckIcon
                  :class="[
                    tier.isStartup ? 'text-[#CEF6F7]' : 'text-primary',
                    'h-6 w-5 flex-none',
                  ]"
                  aria-hidden="true"
                />
                <span :class="[{ 'text-[#CEF6F7]': tier.isStartup }]">
                  {{ feature }}
                </span>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { Switch, SwitchGroup, SwitchLabel } from "@headlessui/vue";
import { CheckIcon } from "@heroicons/vue/20/solid";

const tiers = [
  {
    name: "Autonome",
    id: "tier-freelancer",
    href: "#",
    price: "15€",
    description: "The essentials to provide your best work for clients.",
    features: [
      "Ecosystèmes Open-Source",
      "Services interconnectés",
      "Supervision et alerting",
      "Chiffrement de bout en bout",
      "Uniquement vos ressources",
    ],
    isStartup: false,
    isEntreprise: false,
    alreadyHaveServers: true,
  },
  {
    name: "Startup",
    id: "tier-startup",
    href: "#",
    price: "45€",
    description: "A plan that scales with your rapidly growing business.",
    features: [
      "98% SLA",
      "Support sous 24h par mail",
      "Conformité RGPD",
      "Sauvegarde chez France-Nuage",
      "Failover avec France-Nuage",
    ],
    isStartup: true,
    isEntreprise: false,
    alreadyHaveServers: true,
  },
  {
    name: "Enterprise",
    id: "tier-enterprise",
    href: "#",
    price: "299€",
    description: "Dedicated support and infrastructure for your company.",
    features: [
      "99,9% SLA",
      "Support dédié 24/7 avec réponse sous 4h",
      "Migrations sans iterruptions",
      "Conformité RGPD, ISO 27001",
      "Sauvegarde multi-site",
      "Répartition de charge",
    ],
    isStartup: false,
    isEntreprise: true,
    alreadyHaveServers: true,
  },
  {
    name: "Startup",
    id: "tier-startup",
    href: "#",
    price: "55€",
    description: "A plan that scales with your rapidly growing business.",
    features: [
      "2 vCPU et 4 Go de RAM",
      "98% SLA",
      "Support sous 24h par mail",
      "Conformité RGPD",
      "Sauvegarde",
    ],
    isStartup: true,
    isEntreprise: false,
    alreadyHaveServers: false,
  },
  {
    name: "Enterprise",
    id: "tier-enterprise",
    href: "#",
    price: "299€",
    description: "Dedicated support and infrastructure for your company.",
    features: [
      "8 vCPU et 24 Go de RAM",
      "99,9% SLA",
      "Support dédié 24/7 avec réponse sous 4h",
      "Migrations sans iterruptions",
      "Conformité RGPD, ISO 27001",
      "Sauvegarde multi-site",
    ],
    isStartup: false,
    isEntreprise: true,
    alreadyHaveServers: false,
  },
];

const ownServer = ref(true);
</script>
