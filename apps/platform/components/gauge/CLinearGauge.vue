<template>
  <div class="flex gap-4 items-center">
    <div class="linear-gauge bg-emerald-500"
      :style="`background-image: linear-gradient(to right, transparent ${props.percent | percent}%, #262626 10px)`"
    >
    </div>
    <div>{{ props.percent | percent }} %</div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  totalValue?: number;
  partialValue?: number;
  percent: number;
}

const props = withDefaults(defineProps<Props>(), {
  totalValue: 100,
  partialValue: 0
})

const percent = computed(() => {
  return  (100 * props.partialValue) / props.totalValue
})
</script>

<style lang="scss">
.linear-gauge {
  //background-image: linear-gradient(to right, transparent 0px, #e0e0e0 0px);
  border-radius: 3px;
  display: inline-block;
  height: 8px;
  line-height: 8px;
  width: 180px;
  &:before {
    background-image: linear-gradient(to right, transparent 0px 44px, #777777 44px 45px, transparent 45px 89px, #777777 89px 90px, transparent 90px 134px, #777777 134px 135px, transparent 135px);
    background-size: 180px;
    border-radius: 3px;
    content: "";
    display: inline-block;
    height: 8px;
    vertical-align: top;
    width: 180px;
  }
}
</style>