<template>
  <div
    :class="[
      props.active ? 'ring-2 ring-primary' : 'ring-1 ring-gray-200',
      ' bg-white cursor-pointer hover:ring-primary hover:ring-2 transition-all rounded-md p-3 flex gap-1 items-center',
    ]"
    @click="onClick"
  >
    <div
      class="w-12 [html_:where(&>*)]:w-12 [html_:where(&>*)]:h-8"
      v-html="props.logo"
    />
    <div class="text-center text-md">
      {{ props.name }}
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  name: string;
  logo: string;
  alt: string;
  to?: string;
  active?: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits(["click"]);
const router = useRouter();

const onClick = ($event) => {
  if (props.to) {
    router.push(props.to);
  } else {
    emit("click", $event);
  }
};
</script>

<style scoped></style>
