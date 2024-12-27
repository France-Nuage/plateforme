<template>
  <div class="rounded-lg w-full h-[200px] pl-8 pt-4 pr-8 flex items-center border-2 dark:border-gray-500 select-none" @click="$emit('click')">
    <div class="w-full justify-center flex flex-col">
      <Icon name="solar:add-circle-linear" size="24px" class="text-gray-400 m-auto mb-2" />
      <div class="text-gray-400 text-center">Ajouter une carte</div>
    </div>
  </div>
</template>
