<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  PopoverRoot,
  PopoverTrigger,
  PopoverPortal,
  PopoverContent,
  ListboxRoot,
  ListboxContent,
  ListboxItem,
  ListboxItemIndicator,
} from 'radix-vue'

defineOptions({ inheritAttrs: false })

const props = withDefaults(defineProps<{
  modelValue: string[]
  options: Array<{ value: string; label: string; disabled?: boolean }>
  placeholder?: string
  disabled?: boolean
  id?: string
}>(), {
  placeholder: undefined,
  disabled: false,
  id: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: string[]]
  'blur': [event: FocusEvent]
}>()

const { t } = useI18n()
const inputId = computed(() => props.id ?? `c-multi-select-${Math.random().toString(36).slice(2, 7)}`)
const open = ref(false)

const selectedLabels = computed(() =>
  props.options
    .filter(o => props.modelValue.includes(o.value))
    .map(o => o.label),
)

const displayValue = computed(() => {
  if (selectedLabels.value.length === 0) return ''
  if (selectedLabels.value.length <= 2) return selectedLabels.value.join(', ')
  return t('common.selectedCount', { count: selectedLabels.value.length })
})

function handleUpdate(values: unknown) {
  emit('update:modelValue', Array.isArray(values) ? values.map(String) : [String(values)])
}

function handleBlur(event: FocusEvent) {
  emit('blur', event)
}
</script>

<template>
  <PopoverRoot v-model:open="open">
    <PopoverTrigger
      v-bind="$attrs"
      :id="inputId"
      :disabled="props.disabled"
      class="flex items-center justify-between w-auto min-w-[200px] max-w-full h-[var(--control-height)] px-3 rounded-xl
             bg-surface-container text-on-surface text-md outline-none cursor-pointer
             focus:ring-2 focus:ring-primary/30 focus:bg-surface-container-low
             data-[disabled]:opacity-50 data-[disabled]:cursor-not-allowed
             transition-[background,box-shadow] duration-fast"
      :class="!displayValue ? 'text-on-surface-variant' : ''"
      @blur="handleBlur"
    >
      <span class="truncate">{{ displayValue || (props.placeholder ?? t('common.select')) }}</span>
      <svg
        class="icon-sm text-on-surface-variant shrink-0 ml-2"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M6 9l6 6 6-6" />
      </svg>
    </PopoverTrigger>

    <PopoverPortal>
      <PopoverContent
        align="start"
        :side-offset="4"
        class="min-w-[var(--radix-popover-trigger-width)] max-w-[min(400px,calc(100vw-2rem))] max-h-80 rounded-xl
               bg-surface-container-lowest shadow-md outline-none z-overlay"
      >
        <ListboxRoot
          :model-value="props.modelValue"
          multiple
          @update:model-value="handleUpdate"
        >
          <ListboxContent class="p-1 max-h-72 overflow-y-auto outline-none">
            <ListboxItem
              v-for="opt in props.options"
              :key="opt.value"
              :value="opt.value"
              :disabled="opt.disabled"
              class="flex items-center gap-2 px-3 py-2 text-md text-on-surface rounded-md
                     cursor-pointer outline-none select-none
                     hover:bg-surface-container
                     data-[highlighted]:bg-surface-container
                     data-[disabled]:opacity-40 data-[disabled]:cursor-not-allowed"
            >
              <ListboxItemIndicator class="shrink-0">
                <svg
                  class="icon-xs"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <path d="M20 6L9 17l-5-5" />
                </svg>
              </ListboxItemIndicator>
              <span>{{ opt.label }}</span>
            </ListboxItem>
          </ListboxContent>
        </ListboxRoot>
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>
