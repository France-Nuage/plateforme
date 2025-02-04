<template>
  <div
    :class="[
      props.active ? 'ring-2 ring-primary' : 'ring-1 ring-gray-200',
      ' bg-white cursor-pointer hover:ring-primary hover:ring-2 transition-all rounded-md p-3 flex flex-col gap-1',
    ]"
    @click="onClick"
  >
    <div class="">
      <img
        class="w-10 h-10 aspect-auto mx-auto"
        src="https://scw-marketplace-logos.s3.fr-par.scw.cloud/wordpress.png"
        alt="wordpresss logo"
      />
    </div>
    <div class="text-center text-md">{{ props.name }}</div>
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
