<template>
  <nuxt-layout>
    <div v-if="resource" class="flex flex-col gap-4">
      <h1 class="mb-2">
        Autorisation pour {{ resource.type }}
        {{ navigationStore.$state[resource.type].name }}
      </h1>

      <add-member />

      <c-table
        name="organization_member"
        :headers="headers"
        :data="members"
        @clickRow="clickOnRow"
      >
        <template #col-roles="{ entity, key }">
          <div v-for="role in entity[key]" class="mb-2">
            <c-badge dotted variant="information">{{ role }}</c-badge>
            <br />
          </div>
        </template>
      </c-table>
    </div>

    <show-member ref="memberDrawer" />
  </nuxt-layout>
</template>

<script setup lang="ts">
import CTable from "~/components/table/CTable.vue";
import CBadge from "~/components/badge/CBadge.vue";
import ShowMember from "~/pages/teams/local-components/showMember.vue";
import CButton from "~/components/forms/CButton.vue";
import AddMember from "~/pages/teams/local-components/addMember.vue";

const { resource } = storeToRefs(useNavigationStore());
const navigationStore = useNavigationStore();
const { members } = storeToRefs(useResourceIAMStore());
const { loadMembers } = useResourceIAMStore();
const drawerRef = useTemplateRef("memberDrawer");

const headers = [
  { key: "member", label: "Utilisateur" },
  { key: "roles", label: "Rôles" },
];

const clickOnRow = (item) => {
  console.log(drawerRef, item.value.member);
};

const isOpen = ref(false);

onMounted(() => {
  loadMembers();
});
</script>
