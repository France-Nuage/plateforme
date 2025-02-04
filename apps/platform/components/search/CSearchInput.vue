<template>
  <div class="group relative flex h-12">
    <svg
      viewBox="0 0 20 20"
      fill="none"
      aria-hidden="true"
      class="pointer-events-none absolute left-3 top-0 h-full w-5 stroke-zinc-500"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        d="M12.01 12a4.25 4.25 0 1 0-6.02-6 4.25 4.25 0 0 0 6.02 6Zm0 0 3.24 3.25"
      />
    </svg>
    <input
      type="text"
      :class="[
        `flex-auto w-full focus:outline-none focus:border-none appearance-none bg-transparent pl-10 text-zinc-900 outline-none placeholder:text-zinc-500 dark:border-none focus:flex-none sm:text-sm dark:text-white [&::-webkit-search-cancel-button]:hidden [&::-webkit-search-decoration]:hidden [&::-webkit-search-results-button]:hidden [&::-webkit-search-results-decoration]:hidden`,
      ]"
      @input="$emit('update:modelValue', $event.target.value)"
    />
  </div>
</template>
