<template>
  <c-card class="w-11/12 md:w-8/12 lg:w-4/12 mx-auto mt-24">
    <c-card-header title="Créer un compte" />
    <c-card-body>
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
            description="Le nom d'un compte est semblable à un nom d'une entreprise."
          />
        </div>
      </div>
    </c-card-body>
    <c-card-footer>
      <div>
        <c-button size="sm" variant="filled" @click="router.go(-1)"
          >Annulé</c-button
        >
      </div>
      <div class="flex items-center gap-4">
        <small class="text-gray-600 dark:text-gray-400"
          >Vous pouvez renommer le nom du compte plus tard</small
        >
        <c-button size="sm" @click="onSubmit" :loading="loading"
          >Créer un compte</c-button
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

const { createAccount } = useAccountStore();
const { loadOrganization } = useOrganizationStore();
const { organization } = storeToRefs(useOrganizationStore());
const route = useRoute();
const router = useRouter();
const formData = ref({
  name: "",
});

onMounted(() => {
  loadOrganization(route.params.organizationId);
});

const loading = ref(false);

const onSubmit = () => {
  loading.value = true;
  createAccount({ ...formData.value, organization__id: organization.value.id })
    .then((response) => {
      router.push("/");
    })
    .finally(() => {
      loading.value = false;
    });
};
</script>
