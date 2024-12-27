<template>
  <div :class="`${variantCardStyle[props.variant]} rounded-lg w-full h-[200px] pl-8 pt-4 pr-8`">

    <div class="flex justify-between mt-2 items-center">
      <div class="text-sm">{{ variantTitle[props.variant] }}</div>
      <div>
        <Icon :name="variantLogo[props.variant]" size="20px" class="text-gray-400" />
      </div>
    </div>

    <div class="text-xl mt-4">
      **** 6697
    </div>

  </div>
</template>

<script setup lang="ts">
interface Props {
  variant: 'visa' | 'apple' | 'mastercard'
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'visa'
})

const variantLogo = {
  visa: 'logos:visa',
  apple: 'logos:apple-pay',
  mastercard: 'logos:mastercard'
}

const variantTitle = {
  visa: 'Visa',
  apple: 'Apple Pay',
  mastercard: 'Mastercard'
}

const variantCardStyle = {
  visa: 'bg-blue-500 text-white',
  apple: 'bg-gray-300 text-gray-800',
  mastercard: 'bg-red-500 text-white'
}
</script>

<style scoped>

</style>