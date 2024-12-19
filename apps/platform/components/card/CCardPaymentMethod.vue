<template>
  <div class="rounded-md bg-gray-50 px-6 py-5 sm:flex sm:items-start sm:justify-between bg-gray-800">
    <h4 class="sr-only">Visa</h4>
    <div class="sm:flex sm:items-start">
      <svg class="h-8 w-auto sm:h-6 sm:shrink-0" viewBox="0 0 36 24" aria-hidden="true">
        <rect width="36" height="24" fill="#224DBA" rx="4" />
        <path fill="#fff" d="M10.925 15.673H8.874l-1.538-6c-.073-.276-.228-.52-.456-.635A6.575 6.575 0 005 8.403v-.231h3.304c.456 0 .798.347.855.75l.798 4.328 2.05-5.078h1.994l-3.076 7.5zm4.216 0h-1.937L14.8 8.172h1.937l-1.595 7.5zm4.101-5.422c.057-.404.399-.635.798-.635a3.54 3.54 0 011.88.346l.342-1.615A4.808 4.808 0 0020.496 8c-1.88 0-3.248 1.039-3.248 2.481 0 1.097.969 1.673 1.653 2.02.74.346 1.025.577.968.923 0 .519-.57.75-1.139.75a4.795 4.795 0 01-1.994-.462l-.342 1.616a5.48 5.48 0 002.108.404c2.108.057 3.418-.981 3.418-2.539 0-1.962-2.678-2.077-2.678-2.942zm9.457 5.422L27.16 8.172h-1.652a.858.858 0 00-.798.577l-2.848 6.924h1.994l.398-1.096h2.45l.228 1.096h1.766zm-2.905-5.482l.57 2.827h-1.596l1.026-2.827z" />
      </svg>
      <div class="mt-3 sm:ml-4 sm:mt-0">
        <div class="text-sm font-medium text-gray-900 dark:text-gray-300">Ending with 4242</div>
        <div class="mt-1 text-sm text-gray-600 sm:flex sm:items-center">
          <div class="whitespace-nowrap dark:text-gray-500">Expires 12/20</div>
          <span class="hidden sm:mx-2 sm:inline whitespace-nowrap dark:text-gray-500" aria-hidden="true">&middot;</span>
          <div class="mt-1 sm:mt-0 whitespace-nowrap dark:text-gray-500">Last updated on 22 Aug 2017</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
interface Props {
  small: boolean,
  paymentMethod: {
    id: string;
    last4: string;
    brand: string;
    expiryMonth: string;
    expiryYear: string;
  }
}

const props = defineProps()
</script>