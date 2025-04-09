<template>
  <div>
    <div class="rounded-lg border border-transparent flex flex-col gap-4">
      <slot name="header" />
      <table class="min-w-full divide-y divide-gray-300">
        <thead>
          <tr>
            <th
              scope="col"
              class="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-400 sm:pl-6"
              v-for="(header, i) in headers"
              :key="`${header.key}${i}`"
            >
              <div v-if="header.key === 'select'">
                <p-checkbox
                  v-model:model-value="selectRowAll"
                  :value="selectRowAll"
                  :name="`table_${name}_checkbox_all`"
                />
              </div>
              <span v-else>{{ header.label }}</span>
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          <tr v-for="entity in props.data" :key="`entity-${entity.name}`">
            <td
              class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-normal sm:pl-6 text-gray-600"
              v-for="(header, i) in headers"
              :key="`${header.key}-${i}`"
            >
              <div v-if="header.key === 'select'">
                <p-checkbox
                  v-model="selectRows"
                  :value="entity.id"
                  :name="`table_${name}_checkbox`"
                />
              </div>
              <div
                v-else
                @click="
                  () =>
                    instance?.attrs.onClickRow
                      ? $emit('clickRow', { value: entity })
                      : router.push(`/${props.name}/${entity.id}`)
                "
              >
                <slot
                  :name="`col-${header.key}`"
                  :entity="entity"
                  :key="header.key"
                  :value="_.get(entity, header.key) || '-'"
                >
                  {{ _.get(entity, header.key) || '-' }}
                </slot>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
      <slot name="footer" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import _ from 'lodash'
import { computed, watch, ref, getCurrentInstance } from 'vue'
import { useRouter } from 'vue-router'
import PCheckbox from '@/components/forms/checkbox/PCheckbox.vue'

// todo: implements all supports of this documentation: https://bootstrap-vue.org/docs/components/table#table
interface TableRow {
  [key: string]: CellValue;
}

type CellValue = string | number | boolean | Date | Record<string, unknown> | unknown[] | null | undefined;

interface Props<T extends TableRow = TableRow> {
  headers?: Array<{
    key: string
    label: string
    variant?: string
    sortable?: boolean
  }>
  data?: T[];
  name: string
}

const router = useRouter()
const props = defineProps<Props>()
const headers = computed(
  () =>
    props.headers ||
    (props.data &&
      [...new Set(props.data.flatMap(Object.keys))].map((item) => ({
        label: item,
        key: item,
      }))),
)
const instance = ref(getCurrentInstance())
defineOptions({
  inheritAttrs: false,
})

const selectRowAll = ref(false)
const selectRows = ref([])
watch(selectRowAll, (value) => {
  if (value) {
    selectRows.value = props.data.map((_) => _.id)
  } else {
    selectRows.value = []
  }
})
</script>
