<template>
  <nuxt-layout>
    <h1>Créer une Machine Virtuelle</h1>

    <div class="grid grid-cols-12 divide-x dark:divide-gray-800">
      <div class="col-span-7 pr-8">
        <div class="flex flex-col gap-20 mt-8">
          <informations v-model="formData.name" />
    <!--      <services v-model="formData.service" />-->
<!--          <zones v-model="formData.zone" />-->
    <!--      <instances v-model="formData.instance" />-->
        </div>
      </div>
      <div class="col-start-8 col-end-12 pl-8">
        <price />
      </div>
    </div>

  </nuxt-layout>
</template>

<script setup lang="ts">
import CTitle from "~/components/CTitle.vue";
import Informations from "~/pages/instances/local-components/informations.vue";
import Instances from "~/pages/instances/local-components/instances.vue";
import Services from "~/pages/instances/local-components/services.vue";
import Zones from "~/pages/instances/local-components/zones.vue";
import Price from "~/pages/instances/local-components/price.vue";

const formData = ref({
  name: '',
  type: '',
  service: '',
  instance: '',
  zone: ''
})
</script>


<style scoped>

</style>