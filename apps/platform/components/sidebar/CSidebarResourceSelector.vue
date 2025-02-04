<template>
  <c-dropdown class="w-full">
    <c-dropdown-button class="w-full">
      <div
        class="w-full text-sm font-medium leading-none px-5 py-4 text-left outline-none flex items-center gap-4"
      >
        <div>
          <svg
            class="size-5 text-primary dark:text-gray-200"
            xmlns="http://www.w3.org/2000/svg"
            width="100"
            height="100"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <rect x="9" y="2" width="6" height="6" rx="1" ry="1"></rect>
            <rect x="3" y="14" width="6" height="6" rx="1" ry="1"></rect>
            <rect x="15" y="14" width="6" height="6" rx="1" ry="1"></rect>
            <path d="M12 8v4"></path>
            <path d="M12 12h-6"></path>
            <path d="M12 12h6"></path>
          </svg>
        </div>
        <div>
          <span class="block text-xs text-gray-600 dark:text-gray-400"
            >Organisation</span
          >
          <span class="block font-semibold mt-0.5 text-md dark:text-gray-200">{{
            organization?.name
          }}</span>
        </div>
        <span
          class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2"
        >
          <ChevronUpDownIcon class="h-5 w-5 text-gray-400" aria-hidden="true" />
        </span>
      </div>
    </c-dropdown-button>
    <c-dropdown-list class="w-full" style="width: 400px; left: 1rem">
      <div v-for="curr in organizations" :key="curr.id">
        <div
          class="cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-100 transition-all rounded-lg outline-none"
          @click="onChangeOrganization(curr.id)"
          role="button"
        >
          <div class="flex gap-4 items-center px-2 py-2">
            <span class="font-semibold">{{ curr.name }}</span>
          </div>
        </div>
      </div>
    </c-dropdown-list>
  </c-dropdown>
</template>

<script setup lang="ts">
import { ChevronUpDownIcon } from "@heroicons/vue/20/solid";
import CDropdown from "~/components/dropdown/CDropdown.vue";
import CDropdownButton from "~/components/dropdown/CDropdownButton.vue";
import CDropdownList from "~/components/dropdown/CDropdownList.vue";
import { useNavigationStore } from "~/stores/navigation";

const { organizations, organization } = storeToRefs(useNavigationStore());
const router = useRouter();
const route = useRoute();

const onChangeOrganization = (id) => {
  router.push({ ...route, query: { organization: id } });
};
</script>
