<template>
  <c-dropdown>
    <c-dropdown-button class="-m-1.5 flex items-center p-1.5">
      <span class="sr-only">Open user menu</span>
      <c-avatar />
    </c-dropdown-button>
    <c-dropdown-list class="w-56">

      <div class="py-2 flex items-center gap-2">
        <div class="shrink-0">
          <c-avatar />
        </div>
        <div>
          <span class="w-full block text-sm font-semibold">{{ me.firstname }} {{ me.lastname }}</span>
          <span class="text-sm block">{{ me.email }}</span>
        </div>
      </div>

      <c-divider class="-mx-2 my-2" />

      <c-dropdown-item>
        <template #default v-slot="{ active }">
          <nuxt-link class="h-full w-full block" to="/profile/preferences">Your profile</nuxt-link>
        </template>
      </c-dropdown-item>

<!--      <c-dropdown-item>-->
<!--        <template #default v-slot="{ active }">-->
<!--          <nuxt-link to="/settings/setings" class="h-full w-full block">Account settings</nuxt-link>-->
<!--        </template>-->
<!--      </c-dropdown-item>-->

      <c-divider class="-mx-2 my-2" />

      <c-dropdown-item>
        <template #default v-slot="{ active }">
          <nuxt-link to="/" class="h-full w-full block">Changelog</nuxt-link>
        </template>
      </c-dropdown-item>

      <c-dropdown-item>
        <template #default v-slot="{ active }">
          <nuxt-link to="/" class="h-full w-full block">Support</nuxt-link>
        </template>
      </c-dropdown-item>

      <c-divider class="-mx-2 my-2" />

      <c-dropdown-item>
        <template #default v-slot="{ active }">
          <nuxt-link to="/" class="h-full w-full block text-red-500">Sign out</nuxt-link>
        </template>
      </c-dropdown-item>
    </c-dropdown-list>
  </c-dropdown>
</template>

<script setup lang="ts">
import CDropdownButton from "~/components/dropdown/CDropdownButton.vue";
import CDropdownList from "~/components/dropdown/CDropdownList.vue";
import CDropdown from "~/components/dropdown/CDropdown.vue";
import CDropdownItem from "~/components/dropdown/CDropdownItem.vue";
import CDivider from "~/components/CDivider.vue";
import CAvatar from "~/components/avatar/CAvatar.vue";

const { loadMe } = useAuthStore()
const { me } = storeToRefs(useAuthStore())

onMounted(() => {
  loadMe()
})
</script>
