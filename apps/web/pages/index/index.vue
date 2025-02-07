<template>
  <nuxt-layout>
    <HeroSection />
    <Comparative />
    <MetricsSection />
    <KPISection />
    <CTASection />
    <ApplicationsSection />
    <PricingSection />
    <TeamSection />
  </nuxt-layout>
</template>

<script lang="ts" setup>
import KPISection from "./local-components/KPISection.vue";
import Comparative from "./local-components/Comparative.vue";
import HeroSection from "~/pages/index/local-components/HeroSection.vue";
import MetricsSection from "~/pages/index/local-components/MetricsSection.vue";
import CTASection from "~/pages/index/local-components/CTASection.vue";
import ApplicationsSection from "~/pages/index/local-components/ApplicationsSection.vue";
import PricingSection from "~/pages/index/local-components/PricingSection.vue";
import TeamSection from "~/pages/index/local-components/TeamSection.vue";
</script>
