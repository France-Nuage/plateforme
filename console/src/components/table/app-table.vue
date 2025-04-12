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
              <span>{{ header.label }}</span>
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
              <div
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

<script lang="ts" setup generic="T extends Object[]">
import _ from 'lodash'
import { computed, ref, getCurrentInstance } from 'vue'
import { useRouter } from 'vue-router'

interface Props<T> {
  headers?: Array<{
    key: string
    label: string
    variant?: string
    sortable?: boolean
  }>
  data?: T;
  name: string
}

const props = defineProps<Props<T>>()
const router = useRouter()
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
</script>
