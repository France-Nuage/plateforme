<template>
  <c-card>
    <card-c-card-header title="Informations"> </card-c-card-header>
    <c-card-body>
      <div class="grid grid-cols-12 w-full mb-8 gap-4">
        <div class="col-span-3">
          <c-label label="Donner un nom à votre instance" for="instance-name" />
        </div>
        <div class="col-span-9">
          <c-text-field
            id="instance-name"
            required
            name="instance-name"
            type="text"
            v-model="instanceName"
            placeholder="Nom de l'instance"
            description="Le nom de votre Instance ne peut contenir que des caractères alphanumériques, des points et des tirets."
          />
        </div>
      </div>

      <div class="grid grid-cols-12 gap-4 w-full">
        <div class="col-span-3">
          <c-label label="Choisissez un lieu" for="instance-name" />
        </div>
        <div class="col-span-9">
          <div class="flex gap-4">
            <c-select
              id="instance-region"
              name="instance-region"
              :collections="regions"
              v-model="regionSelected"
              placeholder="Région"
            />
            <c-select
              id="instance-zone"
              name="instance-zone"
              :collections="regionSelected?.zones || []"
              v-model="zoneSelected"
              placeholder="Zones"
            />
          </div>
        </div>
      </div>
    </c-card-body>
  </c-card>
</template>

<script setup lang="ts">
import CTextField from "~/components/forms/CTextField.vue";
import CCard from "~/components/card/CCard.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CLabel from "~/components/forms/CLabel.vue";
import CSelect from "~/components/forms/select/CSelect.vue";
import { useRegionStore } from "~/stores/compute/region";

interface Props {
  modelValue: any;
}

const props = defineProps<Props>();
const regionSelected = ref();
const instanceName = ref("");
const zoneSelected = ref();
const { loadRegions } = useRegionStore();
const { regions } = storeToRefs(useRegionStore());

onMounted(() => {
  loadRegions({ includes: ["zones"] }).then((response) => {
    regionSelected.value = response.data[0];
  });
});

const emit = defineEmits(["update:modelValue"]);

watch(
  () => regionSelected.value,
  (value) => {
    emit("update:modelValue", { ...props.modelValue, regionId: value.id });
  },
);
watch(
  () => instanceName.value,
  (value) => {
    emit("update:modelValue", { ...props.modelValue, name: value });
  },
);
watch(
  () => zoneSelected.value,
  (value) => {
    emit("update:modelValue", { ...props.modelValue, zoneId: value.id });
  },
);
</script>
