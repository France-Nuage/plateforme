<template>
  <nuxt-layout
    name="authentification"
    title="Mot de passe oublié ?"
    v-if="!emailIsSending"
  >
    <p class="mt-2 text-sm/6 text-gray-500 dark:text-gray-400">
      Ne vous inquiéter pas, renseigner votre e-mail, nous vous enverons un lien
      pour le changer !
    </p>

    <div class="mt-10">
      <div>
        <form class="space-y-6" @submit.prevent="onSubmit()">
          <c-text-field
            id="email"
            required
            autocomplete="email"
            name="email"
            type="email"
            label="Email"
            v-model="formData.email"
          />

          <div>
            <c-button type="submit" block :loading="loading"
              >Je valide</c-button
            >
          </div>
        </form>
      </div>
    </div>
  </nuxt-layout>
  <nuxt-layout name="authentification" title="Consulter votre messagerie">
    <p class="dark:text-gray-400 mt-2">
      Merci ! Si l'adresse e-mail de
      <span class="dark:text-primary">{{ formData.email }}</span> figure dans
      notre répertoir utilisateur, vous recevrez un e-mail contenant des
      instructions pour rénitialiser votre mot de passe.
    </p>
    <p class="dark:text-gray-400 mt-2">
      Si vous n'avez pas reçu d'e-mail d'ici 5 minutes, vérifier votre dossier
      de courriers indésirables,
      <nuxt-link to="/auth/forgot-password" class="dark:text-primary"
        >demandez un nouvel envoie</nuxt-link
      >
      ou
      <nuxt-link to="/auth/forgot-password" class="dark:text-primary"
        >réessayer avec une autre adresse e-mail</nuxt-link
      >
      .
    </p>
  </nuxt-layout>
</template>

<script setup lang="ts">
import CTextField from "~/components/forms/CTextField.vue";
import CButton from "~/components/forms/CButton.vue";

const formData = ref({
  email: "",
});
const loading = ref(false);
const emailIsSending = ref(false);
const { resetPasswordRequest } = useAuthStore();

const onSubmit = () => {
  loading.value = true;
  resetPasswordRequest(formData.value)
    .then(() => {
      emailIsSending.value = true;
    })
    .finally(() => {
      loading.value = false;
    });
};
</script>

<style scoped></style>
