<template>
  <main class="h-screen dark:bg-gray-900">
    <div class="flex min-h-full flex-1">
      <div
        class="flex flex-1 flex-col justify-center px-4 py-12 sm:px-6 lg:flex-none lg:px-20 xl:px-24"
      >
        <div class="mx-auto w-full max-w-sm lg:w-96">
          <div>
            <c-logo />
            <h2
              class="mt-8 text-2xl/9 font-bold tracking-tight text-gray-900 dark:text-gray-300"
            >
              {{ props.title }}
            </h2>
          </div>
          <slot />
        </div>
      </div>
      <div class="relative hidden w-0 flex-1 lg:block">
        <img
          class="absolute inset-0 size-full object-cover"
          src="https://images.unsplash.com/photo-1496917756835-20cb06e75b4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=crop&w=1908&q=80"
          alt=""
        />
      </div>
    </div>
    <c-toast-list />
  </main>
</template>

<script setup lang="ts">
import CLogo from "~/components/CLogo.vue";
import CToastList from "~/components/toast/CToastList.vue";

interface Props {
  title: string;
}

const props = defineProps<Props>();
</script>

<style scoped></style>
