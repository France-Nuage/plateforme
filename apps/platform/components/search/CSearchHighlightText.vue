<template>
  <div v-html="highlightedText" />
</template>

<script setup lang="ts">
import { findAll } from "highlight-words-core";

interface Props {
  query: string;
  text: string;
}

const props = defineProps<Props>();

const highlightedText = computed(() => {
  const chunks = findAll({
    caseSensitive: false,
    searchWords: [props.query],
    textToHighlight: props.text,
  });

  return chunks
    .map((chunk) => {
      const { end, highlight, start } = chunk;
      const text = props.text.substr(start, end - start);
      if (highlight) {
        return `<mark>${text}</mark>`;
      } else {
        return text;
      }
    })
    .join("");
});
</script>

<style scoped></style>
