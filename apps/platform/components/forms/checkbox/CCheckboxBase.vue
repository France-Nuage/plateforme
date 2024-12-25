<template>
  <div :class="['w-5 h-5 rounded-md border-2 border-gray-400 flex items-center justify-center', {
    'border-blue-800 bg-primary': props.checked
  }]">
    <svg v-if="props.checked" width="11" height="8" viewBox="0 0 8 6" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M1.72815 3.1313L3.03078 4.43392L6.93866 0.52604C6.93866 0.52604 7.37282 0.0917175 7.80708 0.52604C8.24133 0.960363 7.80708 1.39446 7.80708 1.39446L3.46499 5.73655C3.46499 5.73655 3.03073 6.17066 2.59657 5.73655L0.85973 3.99971C0.85973 3.99971 0.42552 3.56565 0.85973 3.1313C1.29394 2.69694 1.72815 3.1313 1.72815 3.1313Z" fill="white"/>
    </svg>
  </div>
</template>

<script lang="ts" setup>
interface Props {
  checked: boolean
}

const props = defineProps<Props>()
</script>
