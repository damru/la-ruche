<script setup lang="ts">
import { computed } from 'vue'

export interface Tab {
  value: string
  label: string
  count?: number
}

const model = defineModel<string>({ required: true })

const props = defineProps<{
  tabs: Tab[]
  variant?: 'tab' | 'filter'
}>()

const isFilter = computed(() => props.variant === 'filter')
</script>

<template>
  <div
    class="flex overflow-x-auto gap-1"
    :class="isFilter
      ? 'bg-surface-container-low rounded-md p-1 w-max max-w-full overscroll-x-contain filter-scrollbar'
      : 'flex-row pb-px border-b border-outline-variant'"
    :style="!isFilter ? '-webkit-overflow-scrolling: touch; scrollbar-width: none;' : undefined"
  >
    <button
      v-for="tab in props.tabs"
      :key="tab.value"
      type="button"
      class="inline-flex items-center gap-1.5 px-5 py-2 text-sm font-medium whitespace-nowrap
             transition-colors text-on-surface-variant hover:text-on-surface cursor-pointer"
      :class="[
        isFilter ? 'min-w-fit rounded-sm' : 'border-b-2 border-transparent -mb-px',
        model === tab.value
          ? (isFilter ? 'bg-surface-container-lowest text-on-surface shadow-md' : 'border-primary text-primary')
          : ''
      ]"
      @click="model = tab.value"
    >
      {{ tab.label }}
      <span
        v-if="tab.count !== undefined"
        :class="isFilter
          ? 'inline-flex items-center justify-center min-w-[18px] h-[18px] px-[5px] rounded-full bg-primary text-white text-2xs font-semibold'
          : 'text-xs font-semibold text-primary'"
      >{{ tab.count }}</span>
    </button>
  </div>
</template>

<style scoped>
.filter-scrollbar {
  -webkit-overflow-scrolling: touch;
  scrollbar-width: thin;
  scrollbar-color: var(--color-outline-variant) transparent;
}
.filter-scrollbar::-webkit-scrollbar { height: 8px; }
.filter-scrollbar::-webkit-scrollbar-thumb {
  background: var(--color-outline-variant);
  border-radius: var(--radius-full);
}
.filter-scrollbar::-webkit-scrollbar-track { background: transparent; }
@media (hover: none) and (pointer: coarse) {
  .filter-scrollbar { scrollbar-width: none; }
  .filter-scrollbar::-webkit-scrollbar { display: none; }
}
</style>
