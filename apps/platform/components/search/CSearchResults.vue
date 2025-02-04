<template>
  <div v-if="collections.length === 0" class="p-6 text-center dark:bg-gray-900">
    <svg
      viewBox="0 0 20 20"
      fill="none"
      aria-hidden="true"
      class="mx-auto h-5 w-5 stroke-zinc-900 dark:stroke-zinc-600"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        d="M12.01 12a4.237 4.237 0 0 0 1.24-3c0-.62-.132-1.207-.37-1.738M12.01 12A4.237 4.237 0 0 1 9 13.25c-.635 0-1.237-.14-1.777-.388M12.01 12l3.24 3.25m-3.715-9.661a4.25 4.25 0 0 0-5.975 5.908M4.5 15.5l11-11"
      />
    </svg>
    <p class="mt-2 text-xs text-zinc-700 dark:text-zinc-400">
      Nothing found for {{ " " }}
      <strong class="break-words font-semibold text-zinc-900 dark:text-white">
        &lsquo;{{ query }}&rsquo;
      </strong>
      . Please try again.
    </p>
  </div>
  <ul id="search" role="list" v-else>
    <c-search-result
      v-for="(collection, key) in collections"
      :collection="collection"
      :key="key"
      :query="props.query"
    />
  </ul>
</template>

<script setup lang="ts">
import CSearchResult from "~/components/search/CSearchResult.vue";

interface Props {
  collections: Array<any>;
  query: string;
  autocomplete: any;
}

const props = withDefaults(defineProps<Props>(), {
  collections: [],
});

const keyHandler = (e) => {
  const key = e.which || e.keyCode;

  if (key === 38 || (e.shiftKey && key === 9)) {
    handleKeyUp(e);
  } else if (key === 40 || key === 9) {
    handleKeyDown(e);
  } else if (key === 13) {
    handleEnter(e);
  } else if (key === 27) {
    active.value = false;
  }
};

const activeClass = "bg-emerald-400/10";

const handleKeyUp = (e) => {
  e.preventDefault();

  console.log("up");

  const { nodeList, length, currentSelected } = getDomListInformation();

  if (currentSelected === -1 || currentSelected === 0) {
    nodeList[length - 1].classList.add(activeClass);
  }

  if (currentSelected > 0 && currentSelected <= length) {
    nodeList[parseInt(currentSelected) - 1].classList.add(activeClass);
  }

  if (currentSelected !== -1) {
    nodeList[parseInt(currentSelected)].classList.remove(activeClass);
  }
};

const handleKeyDown = (e) => {
  e.preventDefault();

  console.log("down");

  const { nodeList, length, currentSelected } = getDomListInformation();
  if (currentSelected === -1 || currentSelected === length - 1) {
    nodeList[0].classList.add(activeClass);
  }

  if (currentSelected >= 0 && currentSelected < length - 1) {
    nodeList[parseInt(currentSelected) + 1].classList.add(activeClass);
  }

  if (currentSelected !== -1) {
    nodeList[parseInt(currentSelected)].classList.remove(activeClass);
  }
};

const handleEnter = (e) => {
  e.preventDefault();

  const { nodeList, currentSelected } = getDomListInformation();

  if (nodeList[parseInt(currentSelected)]) {
    nodeList[parseInt(currentSelected)].click();
  }
};

const getDomListInformation = () => {
  const target = document.getElementById("search");
  const nodeList = target.querySelectorAll("[role=listitem]");
  const length = nodeList.length;
  const currentSelected =
    length > 0
      ? Array.from(nodeList).findIndex((_) => _.classList.contains(activeClass))
      : -1;

  return {
    nodeList,
    length,
    currentSelected,
  };
};

const addKeyHandler = (e) => {
  console.log("keydown");
  window.addEventListener("keydown", keyHandler);
};

const removeKeyHandler = () => {
  window.removeEventListener("keydown", keyHandler);
};

watch(
  () => props.collections,
  (value) => {
    // TODO: fix this, as the key handler is never removed, which makes the keys to move 2 by 2
    value ? addKeyHandler() : removeKeyHandler();
  },
  { immediate: true },
);
</script>

<style scoped></style>
