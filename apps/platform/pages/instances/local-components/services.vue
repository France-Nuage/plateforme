<template>
  <c-card>
    <c-card-header
      title="Choisissez un service"
      description="Le service qui tourne sur votre Instance. Il peut s'agir d'un OS, d'une InstantApp ou d'une de vos images personnalisées."
    />
    <c-card-body>
      <div class="grid grid-cols-12 w-full">
        <div class="col-span-3"></div>
        <div class="col-span-9">
          <fieldset aria-label="Server service">
            <RadioGroup
              v-model="selected"
              class="space-y-4"
              name="zones_radio_group"
              id="zones_radio_group"
            >
              <ul role="list" class="grid grid-cols-4 gap-6">
                <RadioGroupOption
                  as="template"
                  v-for="service in services"
                  :key="service.name"
                  :value="service"
                  :aria-label="service.name"
                  :aria-description="`${service.name}`"
                  v-slot="{ active, checked }"
                >
                  <c-service-square
                    :active="checked"
                    :name="service.name"
                    logo=""
                    :alt="service.name"
                  />
                </RadioGroupOption>
              </ul>
            </RadioGroup>
          </fieldset>
        </div>
      </div>
    </c-card-body>
  </c-card>
</template>

<script setup lang="ts">
import CCard from "~/components/card/CCard.vue";
import CCardHeader from "~/components/card/CCardHeader.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import { RadioGroup, RadioGroupOption } from "@headlessui/vue";
import CServiceSquare from "~/components/services/CServiceSquare.vue";

const selected = ref();
const services = [
  {
    name: "Wordpress",
  },
  {
    name: "Penpot",
  },
  {
    name: "Grafana",
  },
  {
    name: "Custom",
  },
];
</script>

<style scoped></style>
