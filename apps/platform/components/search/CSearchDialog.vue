<template>
  <c-modal v-model="dialogIsOpen" no-header no-footer>
    <div>
      <form action="">
        <c-search-input v-model="searchTerms" />
        <div class="border-t border-zinc-200 bg-white empty:hidden dark:border-none dark:bg-white/2.5">
          <c-search-results
            v-if="searchTerms.length > 0"
            :collections="collections.slice(0, 5)"
            :autocomplete="searchTerms"
            :query="searchTerms"
          />
        </div>
      </form>
    </div>
  </c-modal>
</template>

<script setup lang="ts">
import CSearchInput from "~/components/search/CSearchInput.vue";
import CModal from "~/components/modal/CModal.vue";
import CSearchResults from "~/components/search/CSearchResults.vue";

interface Props {
  modelValue: boolean
}

const emit = defineEmits(['update:modelValue', 'close'])

const props = defineProps<Props>()
const searchTerms = ref('')
const collections = ref([])
const dialogIsOpen = ref(false)
const onKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'k' && (event.metaKey || event.ctrlKey)) {
    event.preventDefault()
    emit('update:modelValue', true)
  }
}

onMounted(async () => {
  if (props.modelValue) {
    return
  }

  window.addEventListener('keydown', onKeyDown)
})


onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
})

const onClose = () => {
  searchTerms.value = ''
  emit('close')
}


watch(searchTerms, (query) => {
  collections.value = []

  if (query.length === 0) {
    collections.value = []
  }
})

watch(() => props.modelValue, () => {
  dialogIsOpen.value = props.modelValue
}, { immediate: true })

watch(dialogIsOpen, () => {
  emit('update:modelValue', dialogIsOpen.value)
}, { immediate: true })
</script>
