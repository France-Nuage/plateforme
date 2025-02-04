<template>
  <nuxt-layout name="authentification" title="Rénitialisation du mot de passe">
    <form class="space-y-6" @submit.prevent="onSubmit">
      <c-text-field
        v-model.trim="formData.password"
        id="password"
        required
        autocomplete="password"
        name="password"
        type="password"
        label="Mot de passe"
      />
      <c-text-field
        v-model.trim="confirmPassword"
        id="confirm-password"
        required
        autocomplete="password"
        name="confirm-password"
        type="password"
        label="Confirmer votre mot de passe"
      />

      <div>
        <c-button type="submit" block :loading="loading">
          Se connecter
        </c-button>
      </div>
    </form>
  </nuxt-layout>
</template>

<script setup lang="ts">
import CTextField from "~/components/forms/CTextField.vue";
import CButton from "~/components/forms/CButton.vue";

const formData = ref({
  password: "",
});

const router = useRouter();
const route = useRoute();
const confirmPassword = ref("");
const loading = ref(false);
const { resetPassword } = useAuthStore();

const onSubmit = () => {
  if (confirmPassword.value === formData.value.password) {
    loading.value = true;
    resetPassword({
      password: formData.value.password,
      token: route.query.token,
    })
      .then(() => {
        router.push("/auth/login");
      })
      .finally(() => {
        loading.value = false;
      });
  }
};
</script>

<style scoped></style>
