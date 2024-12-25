<template>
  <nuxt-layout>
    <div class="flex flex-col gap-8">
      <div v-if="instance">
        <h1>Instance ({{ instance.name }})</h1>
      </div>

      <div class="grid grid-cols-12">

        <div class="col-span-6 flex flex-col gap-8">
          <c-card>
            <c-card-header title="Informations"  />
            <c-card-body>

              <div class="flex justify-between pb-4">
                <div>
                  <div class="font-semibold">Statut</div>
                  <div class="flex items-center gap-3">
                    <c-pulsing-dot-loader />
                    <div class="dark:text-gray-400">En cours</div>
                  </div>
                </div>

                <div>
                  <div class="font-semibold">Type</div>
                  <div class="dark:text-gray-400">En cours</div>
                </div>

                <div>
                  <div class="font-semibold">Image d'origine</div>
                  <div class="dark:text-gray-400">En cours</div>
                </div>

                <div>
                  <div class="font-semibold">Zone de disponibilité</div>
                  <div class="dark:text-gray-400">En cours</div>
                </div>

              </div>

              <c-divider />

              <div class="flex gap-2 py-4">
                <div class="font-semibold">Commande SSH</div>
                <div class="dark:text-gray-400">ssh root@51.15.39.8</div>
              </div>

              <c-divider />

            </c-card-body>
          </c-card>

          <c-card>
            <c-card-header title="Zone de danger"  />
            <c-card-body>
              <c-alert title="Request for organization deletion" variant="danger">
                <div class="mb-3">
                  <span class="text-red-300 font-semibold">Cette action supprimera tous les volumes et données de ce serveur de stockage.</span>
                  Vous ne pouvez supprimer que les Instances démarrées ou arrêtées.
                  Réalisez régulièrement des snapshots pour éviter de perdre des données.
                </div>

                <c-button variant="danger" size="sm">Request to delete organization</c-button>
              </c-alert>
            </c-card-body>
          </c-card>
        </div>

      </div>

    </div>
  </nuxt-layout>
</template>

<script setup lang="ts">
import CAlert from "~/components/alert/CAlert.vue";
import CCardHeader from "~/components/card/CCardHeader.vue";
import CCard from "~/components/card/CCard.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CButton from "~/components/forms/CButton.vue";
import CPulsingDotLoader from "~/components/loader/CPulsingDotLoader.vue";

const { instance } = storeToRefs(useInstanceStore());
const { loadInstance } = useInstanceStore();
const route = useRoute()

onMounted(() => {
  loadInstance(route.params.id);
})
</script>
