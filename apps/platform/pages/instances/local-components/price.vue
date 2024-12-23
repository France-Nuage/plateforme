<template>
  <div class="bg-white shadow sm:rounded-lg dark:bg-gray-900">
    <div class="px-4 py-5 sm:p-6">
      <div class="mb-6">
        <h2 class="text-2xl">Estimation mentuelle</h2>
        <p class="text-gray-600 dark:text-gray-400 text-sm">Vous payez ce que vous consommez : facturation à la seconde, sans frais initiaux.</p>
      </div>
      <h3 class="text-base font-semibold text-gray-900 dark:text-gray-200">Payment method</h3>
      <div class="mt-5">
        <div class="rounded-md bg-gray-50 px-6 py-5 sm:flex sm:items-start sm:justify-between bg-blue-50 dark:bg-gray-800">
          <h4 class="sr-only">Visa</h4>
          <div class="sm:flex sm:items-start">
            <svg class="h-8 w-auto sm:h-6 sm:shrink-0" viewBox="0 0 36 24" aria-hidden="true">
              <rect width="36" height="24" fill="#224DBA" rx="4" />
              <path fill="#fff" d="M10.925 15.673H8.874l-1.538-6c-.073-.276-.228-.52-.456-.635A6.575 6.575 0 005 8.403v-.231h3.304c.456 0 .798.347.855.75l.798 4.328 2.05-5.078h1.994l-3.076 7.5zm4.216 0h-1.937L14.8 8.172h1.937l-1.595 7.5zm4.101-5.422c.057-.404.399-.635.798-.635a3.54 3.54 0 011.88.346l.342-1.615A4.808 4.808 0 0020.496 8c-1.88 0-3.248 1.039-3.248 2.481 0 1.097.969 1.673 1.653 2.02.74.346 1.025.577.968.923 0 .519-.57.75-1.139.75a4.795 4.795 0 01-1.994-.462l-.342 1.616a5.48 5.48 0 002.108.404c2.108.057 3.418-.981 3.418-2.539 0-1.962-2.678-2.077-2.678-2.942zm9.457 5.422L27.16 8.172h-1.652a.858.858 0 00-.798.577l-2.848 6.924h1.994l.398-1.096h2.45l.228 1.096h1.766zm-2.905-5.482l.57 2.827h-1.596l1.026-2.827z" />
            </svg>
            <div class="mt-3 sm:ml-4 sm:mt-0">
              <div class="text-sm font-medium text-gray-900 dark:text-gray-100">Ending with 4242</div>
              <div class="mt-1 text-sm text-gray-600 sm:flex sm:items-center dark:text-gray-400 whitespace-nowrap">
                <div>Expires 12/20</div>
                <span class="hidden sm:mx-2 sm:inline" aria-hidden="true">&middot;</span>
                <div class="mt-1 sm:mt-0">Last updated on 22 Aug 2017</div>
              </div>
            </div>
          </div>
          <div class="mt-4 sm:ml-6 sm:mt-0 sm:shrink-0">
            <c-button size="sm">Modifier</c-button>
          </div>
        </div>
      </div>
    </div>
    <section aria-labelledby="summary-heading" class="mt-16 rounded-lg bg-gray-50 px-4 py-6 sm:p-6 lg:col-span-5 lg:mt-0 lg:p-8">
      <h2 id="summary-heading" class="text-lg font-medium text-gray-900 dark:text-gray-200">Order summary</h2>

      <dl class="mt-6 space-y-4">
        <div class="flex items-center justify-between">
          <dt class="text-sm text-gray-600 dark:text-gray-400">Subtotal</dt>
          <dd class="text-sm font-medium text-gray-900 dark:text-gray-200">€99.00</dd>
        </div>
        <div class="flex items-center justify-between border-t border-gray-200 dark:border-gray-800 pt-4">
          <dt class="flex items-center text-sm text-gray-600 dark:text-gray-400">
            <span>Shipping estimate</span>
            <a href="#" class="ml-2 shrink-0 text-gray-400 hover:text-gray-500">
              <span class="sr-only">Learn more about how shipping is calculated</span>
              <QuestionMarkCircleIcon class="size-5" aria-hidden="true" />
            </a>
          </dt>
          <dd class="text-sm font-medium text-gray-900 dark:text-gray-200">€5.00</dd>
        </div>
        <div class="flex items-center justify-between border-t border-gray-200 dark:border-gray-800 pt-4">
          <dt class="flex text-sm text-gray-600 dark:text-gray-400">
            <span>Tax estimate</span>
            <a href="#" class="ml-2 shrink-0 text-gray-400 hover:text-gray-500">
              <span class="sr-only">Learn more about how tax is calculated</span>
              <QuestionMarkCircleIcon class="size-5" aria-hidden="true" />
            </a>
          </dt>
          <dd class="text-sm font-medium text-gray-900 dark:text-gray-200">€8.32</dd>
        </div>
        <div class="flex items-center justify-between border-t border-gray-200 dark:border-gray-800 pt-4">
          <dt class="text-base font-medium text-gray-900 dark:text-gray-200">Total</dt>
          <dd class="text-base font-medium text-gray-900 dark:text-gray-200">€112.32</dd>
        </div>
      </dl>

      <div class="mt-6">
        <c-button block @click.stop="$emit('click')" :loading="props.loading">Commander</c-button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { QuestionMarkCircleIcon } from "@heroicons/vue/16/solid";
import CButton from "~/components/forms/CButton.vue";

interface Props {
  loading: boolean
}

const props = defineProps<Props>()
</script>