<template>
  <nuxt-layout>
    <div v-if="instances.length">

      <div class="mb-8 flex justify-between">
        <h1>Liste de vos instances</h1>
        <c-button @click="$router.push('/instances/create')" size="sm">Créer une instance</c-button>
      </div>

      <c-table name="instances" :data="instances" :headers="headers">

        <template #col-status>
          <div class="flex gap-4 items-center">
            <c-pulsing-dot-loader />
            En cours
          </div>
        </template>

        <template #col-node.cluster.zone="{ entity, key, value }">
          {{ value.name }}
        </template>

        <template #col-action="{ entity }">

          <c-dropdown @click.stop="">
            <c-dropdown-button>
              <Icon name="solar:menu-dots-bold" size="24" />

            </c-dropdown-button>
            <c-dropdown-list>
              <c-dropdown-item>Plus d'information</c-dropdown-item>
              <c-dropdown-divider />
              <c-dropdown-item>Démarrer</c-dropdown-item>
              <c-dropdown-item>redémarrer</c-dropdown-item>
              <c-dropdown-item>éteindre</c-dropdown-item>
            </c-dropdown-list>
          </c-dropdown>

        </template>

      </c-table>

    </div>
    <empty-screen v-else />
  </nuxt-layout>
</template>

<script setup lang="ts">
import EmptyScreen from "~/pages/instances/local-components/empty-screen.vue";
import CTable from "~/components/table/CTable.vue";
import CDropdown from "~/components/dropdown/CDropdown.vue";
import CDropdownDivider from "~/components/dropdown/CDropdownDivider.vue";
import CDropdownList from "~/components/dropdown/CDropdownList.vue";
import CDropdownItem from "~/components/dropdown/CDropdownItem.vue";
import CDropdownButton from "~/components/dropdown/CDropdownButton.vue";
import CButton from "~/components/forms/CButton.vue";
import CPulsingDotLoader from "~/components/loader/CPulsingDotLoader.vue";

const { instances } = storeToRefs(useInstanceStore());
const { loadInstances } = useInstanceStore();
const interval = ref();
const headers = [
  { key: 'select', label: 'Nom' },
  { key: 'status', label: 'Status' },
  { key: 'name', label: 'Nom' },
  { key: 'createdAt', label: 'Création' },
  { key: 'ip', label: 'IP' },
  { key: 'node.cluster.zone', label: 'Zone' },
  { key: 'action', label: 'Actions' },
]

const loadInstancesFromStore = () => {
  return loadInstances({ includes: ['node.cluster.zone'] });
}

onMounted(() => {
  loadInstancesFromStore();
  interval.value = setInterval(() => {
    loadInstancesFromStore();
  }, 5000);
})

onUnmounted(() => {
  clearInterval(interval.value);
})
</script>

