<template>
  <div class="">
    <c-alert title="Aucun moyen de paiments" variant="information">
      <div class="mb-3">
        Vous n'avez aucun moyen de paiement enregistrer, vous devez en ajouter un si vous voulez utiliser nos services.
      </div>

      <c-button variant="information" size="sm">Ajouter un moyen de paiements</c-button>
    </c-alert>

    <div class="mt-4">
      <c-table />
    </div>
  </div>
</template>

<script setup lang="ts">
import CAlert from "~/components/alert/CAlert.vue";
import CButton from "~/components/forms/CButton.vue";
import CTable from "~/components/table/CTable.vue";
</script>

<style scoped>

</style>