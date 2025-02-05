<template>
  <nuxt-layout name="authentification" title="S'inscrire">
    <p class="mt-2 text-sm/6 text-gray-500">
      Déjà membre ? {{ " " }}
      <nuxt-link to="/auth/login" class="font-semibold text-primary"
        >Je me connecte !</nuxt-link
      >
    </p>

    <div class="mt-10">
      <div>
        <!--        <c-social-button social-provider="google" />-->
        <!--        <c-separator class="my-10" label="Ou" />-->

        <form class="space-y-6" @submit.prevent="onSubmit">
          <div class="flex flex-col md:flex-row gap-4">
            <c-text-field
              v-model.trim="formData.lastname"
              id="lastname"
              required
              autocomplete="lastname"
              name="lastname"
              type="text"
              label="Votre nom"
            />
            <c-text-field
              v-model.trim="formData.firstname"
              id="firstname"
              required
              autocomplete="firstname"
              name="firstname"
              type="text"
              label="Votre prénom"
            />
          </div>

          <c-text-field
            v-model.trim="formData.email"
            id="email"
            required
            autocomplete="email"
            name="email"
            type="email"
            label="E-mail"
          />
          <c-text-field
            v-model.trim="formData.password"
            id="password"
            required
            autocomplete="password"
            name="password"
            type="password"
            label="Mot de passe"
          />

          <div>
            <c-button
              type="submit"
              block
              :loading="loading"
              data-id="subscribe-submit"
            >
              S'inscrire
            </c-button>
          </div>
        </form>
      </div>
    </div>
  </nuxt-layout>
</template>

<script setup lang="ts">
import CTextField from "~/components/forms/CTextField.vue";
import CSocialButton from "~/components/autentification/CSocialButton.vue";
import CButton from "~/components/forms/CButton.vue";
import CSeparator from "~/components/CSeparator.vue";

const formData = ref({
  lastname: "",
  firstname: "",
  email: "",
  password: "",
});

const { subscribe } = useAuthStore();
const { authenticated } = storeToRefs(useAuthStore());
const router = useRouter();
const loading = ref(false);

const onSubmit = () => {
  loading.value = true;
  subscribe(formData.value).finally(() => {
    if (authenticated.value) {
      router.push("/");
    }
    loading.value = false;
  });
};
</script>
