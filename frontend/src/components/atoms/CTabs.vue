<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

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

const containerClass = computed(() => (isFilter.value ? 'tabs--filter filter-scrollbar' : 'tabs--tab'))

const containerEl = ref<HTMLElement | null>(null)
const fadeLeft = ref(false)
const fadeRight = ref(false)

/**
 * scrollWidth can exceed scrollLeft + clientWidth by subpixel rounding,
 * hence the 1px epsilon before declaring an overflow side.
 */
function updateFades(): void {
  const el = containerEl.value
  if (!el) return
  fadeLeft.value = el.scrollLeft > 1
  fadeRight.value = el.scrollLeft + el.clientWidth < el.scrollWidth - 1
}

let observer: ResizeObserver | undefined
onMounted(() => {
  if (!isFilter.value) return
  updateFades()
  observer = new ResizeObserver(updateFades)
  if (containerEl.value) observer.observe(containerEl.value)
})
onBeforeUnmount(() => observer?.disconnect())
watch(
  () => props.tabs,
  () => {
    if (isFilter.value) nextTick(updateFades)
  },
)

const tabClass = (isActive: boolean) => ({
  'tabs__button': true,
  'tabs__button--filter': isFilter.value,
  'tabs__button--tab': !isFilter.value,
  'tabs__button--active-filter': isFilter.value && isActive,
  'tabs__button--active-tab': !isFilter.value && isActive,
})

const countClass = (isActive: boolean) => ({
  'tabs__count': true,
  'tabs__count--filter': isFilter.value,
  'tabs__count--tab': !isFilter.value,
  'tabs__count--active-tab': !isFilter.value && isActive,
})
</script>

<template>
  <div>
    <div class="tabs__outer">
      <div
        ref="containerEl"
        :class="[
          'tabs',
          containerClass,
          {
            'tabs--fade-left': isFilter && fadeLeft,
            'tabs--fade-right': isFilter && fadeRight,
          },
        ]"
        @scroll.passive="updateFades"
      >
        <button
          v-for="tab in props.tabs"
          :key="tab.value"
          type="button"
          :class="tabClass(model === tab.value)"
          @click="model = tab.value"
        >
          {{ tab.label }}
          <span
            v-if="tab.count !== undefined"
            :class="countClass(model === tab.value)"
          >{{ tab.count }}</span>
        </button>
      </div>
      <span v-if="isFilter && fadeRight" class="tabs__hint tabs__hint--right" aria-hidden="true">
        <CIconChevronRight />
      </span>
      <span v-if="isFilter && fadeLeft" class="tabs__hint tabs__hint--left" aria-hidden="true">
        <CIconChevronRight />
      </span>
    </div>
    <slot></slot>
  </div>
</template>

<style scoped>
.tabs {
  display: flex;
  gap: var(--space-1);
  overflow-x: auto;
  margin-bottom: var(--space-4);
}

.tabs--tab {
  flex-direction: row;
  padding-bottom: 1px;
  border-bottom: 1px solid var(--color-outline-variant);
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
  overflow:visible;
}

.tabs--filter {
  width: max-content;
  max-width: 100%;
  padding: var(--space-1);
  border-radius: var(--radius-xl);
  background: var(--color-surface-container-low);
  overscroll-behavior-x: contain;
}

.tabs__button {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
  white-space: nowrap;
  color: var(--color-on-surface-variant);
  cursor: pointer;
  transition: var(--transition-fast);
}

.tabs__button:hover {
  color: var(--color-on-surface);
}

.tabs__button--tab {
  margin-bottom: -2px;
  border-bottom: 4px solid transparent;
}

.tabs__button--filter {
  min-height: var(--control-height);
  min-width: fit-content;
  border-radius: var(--radius-lg);
}

.tabs__button--active-tab {
  border-bottom-color: var(--color-primary);
  color: var(--color-primary);
}

.tabs__button--active-filter {
  color: var(--color-on-surface);
  background: var(--color-surface-container-lowest);
  box-shadow: var(--shadow-md);
}

.tabs__count {
  font-weight: var(--fw-semibold);
}

.tabs__count--tab {
  font-size: var(--text-xs);
  color: var(--color-primary);
}

.tabs__count--active-tab {
  color: var(--color-primary);
}

.tabs__count--filter {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: var(--icon-md);
  height: var(--icon-md);
  padding: 0 var(--space-1);
  border-radius: var(--radius-full);
  background: var(--color-primary);
  color: var(--color-on-primary);
  font-size: var(--text-2xs);
}

.tabs__outer {
  position: relative;
}

.tabs__hint {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: var(--icon-lg);
  height: var(--icon-lg);
  border-radius: var(--radius-full);
  background: var(--color-surface-container-lowest);
  color: var(--color-on-surface-variant);
  box-shadow: var(--shadow-md);
  pointer-events: none;
  z-index: var(--z-raised);
}

.tabs__hint svg {
  width: var(--icon-sm);
  height: var(--icon-sm);
}

.tabs__hint--right {
  right: var(--space-2);
}

.tabs__hint--left {
  left: var(--space-2);
}

.tabs__hint--left svg {
  transform: rotate(180deg);
}

.tabs--fade-left {
  mask-image: linear-gradient(to right, transparent, black 24px);
}

.tabs--fade-right {
  mask-image: linear-gradient(to right, black calc(100% - 24px), transparent);
}

.tabs--fade-left.tabs--fade-right {
  mask-image: linear-gradient(to right, transparent, black 24px, black calc(100% - 24px), transparent);
}

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
