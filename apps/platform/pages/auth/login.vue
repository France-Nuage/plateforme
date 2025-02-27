<template>
  <nuxt-layout name="authentification" title="Se connecter">
    <p class="mt-2 text-sm/6 text-gray-500">
      Pas encore membre ? {{ " " }}
      <nuxt-link
        to="/auth/subscribe"
        class="font-semibold text-primary"
        data-pw="subscribe-link"
        >Inscrivez-vous !</nuxt-link
      >
    </p>

    <div class="mt-10">
      <div>
        <!--        <c-social-button social-provider="google" />-->
        <!--        <c-separator class="my-10" label="Ou" />-->

        <form class="space-y-6" @submit.prevent="onSubmit">
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

          <div class="flex items-center justify-between">
            <div class="text-sm/6">
              <nuxt-link
                to="/auth/forgot-password"
                class="font-semibold text-primary"
                data-pw="forgot-password-link"
                >Mot de passe oublié ?</nuxt-link
              >
            </div>

            <div class="flex items-center" />
          </div>

          <div>
            <c-button
              type="submit"
              block
              :loading="loading"
              data-id="login-submit"
            >
              Se connecter
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

const { authenticate } = useAuthStore();
const { authenticated } = storeToRefs(useAuthStore());

const formData = ref({
  email: "",
  password: "",
});

const router = useRouter();
const loading = ref(false);

const onSubmit = () => {
  loading.value = true;
  authenticate(formData.value).finally(() => {
    if (authenticated) {
      router.push("/");
    }
    loading.value = false;
  });
};
</script>
