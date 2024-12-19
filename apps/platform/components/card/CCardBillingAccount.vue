<template>
  <div class="border border-dotted dark:border-gray-600 rounded-lg p-4">
    <div>
      <span class="block text-sm font-semibold">Mon compte de facturation</span>
    </div>
    <div class="mt-4">
      <div class="flex gap-4">
        <svg class="h-8 w-auto sm:h-6 sm:shrink-0" viewBox="0 0 36 24" aria-hidden="true">
          <rect width="36" height="24" fill="#224DBA" rx="4" />
          <path fill="#fff" d="M10.925 15.673H8.874l-1.538-6c-.073-.276-.228-.52-.456-.635A6.575 6.575 0 005 8.403v-.231h3.304c.456 0 .798.347.855.75l.798 4.328 2.05-5.078h1.994l-3.076 7.5zm4.216 0h-1.937L14.8 8.172h1.937l-1.595 7.5zm4.101-5.422c.057-.404.399-.635.798-.635a3.54 3.54 0 011.88.346l.342-1.615A4.808 4.808 0 0020.496 8c-1.88 0-3.248 1.039-3.248 2.481 0 1.097.969 1.673 1.653 2.02.74.346 1.025.577.968.923 0 .519-.57.75-1.139.75a4.795 4.795 0 01-1.994-.462l-.342 1.616a5.48 5.48 0 002.108.404c2.108.057 3.418-.981 3.418-2.539 0-1.962-2.678-2.077-2.678-2.942zm9.457 5.422L27.16 8.172h-1.652a.858.858 0 00-.798.577l-2.848 6.924h1.994l.398-1.096h2.45l.228 1.096h1.766zm-2.905-5.482l.57 2.827h-1.596l1.026-2.827z" />
        </svg>
        <div class="flex flex-col">
          <span class="block text-sm dark:text-gray-200">Ending with 4242</span>
<!--          <span class="hidden sm:mx-2 sm:inline whitespace-nowrap dark:text-gray-500" aria-hidden="true">&middot;</span>-->
          <span class="block text-sm dark:text-gray-400">Expires 12/20</span>
        </div>
      </div>
    </div>
    <div class="mt-4">

      <div class="text-center text-sm dark:text-gray-400">Lié à aucun projet</div>

    </div>
  </div>
</template>

<script setup lang="ts">
import CCardPaymentMethod from "~/components/card/CCardPaymentMethod.vue";

interface Props {
  billing: any
}

const props = defineProps<Props>()
const paymentProfile = ref(null)
</script>
