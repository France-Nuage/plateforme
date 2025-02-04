<template>
  <c-card class="w-11/12 md:w-8/12 lg:w-4/12 mx-auto mt-24">
    <c-card-header title="Créer une nouvelle organisation" />
    <c-card-body>
      <p class="text-xs mb-8 dark:text-gray-400">
        <span class="font-semibold"
          >This is your organization within France-Nuage.</span
        >
        For example, you can use the name of your company or department.
      </p>

      <div class="flex flex-col gap-8">
        <div class="grid grid-cols-12 w-full">
          <c-label label="Nom" for="name" class="col-span-3" />
          <c-text-field
            v-model="formData.name"
            id="name"
            required
            name="name"
            type="text"
            class="col-span-9"
            description="Quel serait la meilleur description pour votre organisation ?"
          />
        </div>
      </div>
    </c-card-body>
    <c-card-footer>
      <div>
        <CButton variant="filled" size="sm" @click="router.go(-1)"
          >Annulé</CButton
        >
      </div>
      <div class="flex items-center gap-4">
        <small class="text-xs dark:text-gray-400"
          >Vous pouvez renommer le nom <br />
          de l'organisation plus tard</small
        >
        <CButton size="sm" @click="onSubmit" :loading="loading"
          >Créer une organisation</CButton
        >
      </div>
    </c-card-footer>
  </c-card>
</template>

<script setup lang="ts">
import CCardBody from "~/components/card/CCardBody.vue";
import CButton from "~/components/forms/CButton.vue";
import CCardFooter from "~/components/card/CCardFooter.vue";
import CCardHeader from "~/components/card/CCardHeader.vue";
import CCard from "~/components/card/CCard.vue";
import CTextField from "~/components/forms/CTextField.vue";
import CLabel from "~/components/forms/CLabel.vue";
import CSwitch from "~/components/forms/CSwitch.vue";
import CSelect from "~/components/forms/CSelect.vue";

const plans = [
  { id: 1, name: "Durward Reynolds", price: "" },
  { id: 2, name: "Kenton Towne", price: "" },
  { id: 3, name: "Therese Wunsch", price: "" },
  { id: 4, name: "Benedict Kessler", price: "" },
  { id: 5, name: "Katelyn Rohan", price: "" },
];
const formData = ref({
  name: "",
  ownServer: false,
  selectedPlan: plans[0],
});
const router = useRouter();
const { createOrganization } = useOrganizationStore();
const { organization } = storeToRefs(useOrganizationStore());
const loading = ref(false);

const onSubmit = () => {
  loading.value = true;
  createOrganization(formData.value)
    .then(() => {
      router.push({ path: `/onboarding/new/${organization.value.id}` });
    })
    .finally(() => {
      loading.value = false;
    });
};
</script>
