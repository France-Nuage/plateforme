<template>
  <div
    class="w-full pointer-events-auto overflow-hidden rounded-lg bg-white shadow-lg ring-1 ring-black/5"
  >
    <div class="p-4">
      <div class="flex items-start">
        <div class="shrink-0">
          <Icon
            v-if="props.type === 'danger'"
            name="solar:danger-circle-linear"
            class="text-red-400"
            size="24px"
          />
          <Icon
            v-if="props.type === 'success'"
            name="solar:check-circle-linear"
            class="text-green-400"
            size="24px"
          />
          <Icon
            v-if="props.type === 'warning'"
            name="solar:shield-warning-linear"
            class="text-orange-400"
            size="24px"
          />
        </div>
        <div class="ml-3 w-0 flex-1 pt-0.5">
          <div class="text-sm font-medium text-gray-900">{{ props.title }}</div>
          <div v-if="props.description" class="mt-1 text-sm text-gray-500">
            {{ props.description }}
          </div>
        </div>
        <!--        <div class="ml-4 flex shrink-0">-->
        <!--          <button type="button" class="inline-flex rounded-md bg-white text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2">-->
        <!--            <span class="sr-only">Close</span>-->
        <!--            x-->
        <!--          </button>-->
        <!--        </div>-->
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  type?: "danger" | "success" | "warning";
  title: string;
  description: string;
}

const props = withDefaults(defineProps<Props>(), {
  type: "danger",
});

// https://dribbble.com/shots/23230014-Alerts-in-Hashnode
</script>
