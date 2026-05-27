<script setup lang="ts">
export interface Tab {
  value: string
  label: string
  count?: number
}

const model = defineModel<string>({ required: true })

const props = defineProps<{
  tabs: Tab[]
}>()
</script>

<template>
  <div
    class="flex flex-row overflow-x-auto gap-1 pb-px border-b border-outline-variant"
    style="-webkit-overflow-scrolling: touch; scrollbar-width: none;"
  >
    <button
      v-for="tab in props.tabs"
      :key="tab.value"
      type="button"
      class="flex items-center gap-1.5 px-5 py-2 text-sm font-medium whitespace-nowrap
             transition-colors text-on-surface-variant hover:text-on-surface
             border-b-2 border-transparent -mb-px"
      :class="model === tab.value
        ? 'border-primary text-primary'
        : ''"
      @click="model = tab.value"
    >
      {{ tab.label }}
      <span
        v-if="tab.count !== undefined"
        class="text-xs font-semibold text-primary"
      >{{ tab.count }}</span>
    </button>
  </div>
</template>
