<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFieldValidation } from '@/composables/useFieldValidation'
import {
  SelectRoot, SelectTrigger, SelectValue, SelectPortal,
  SelectContent, SelectViewport, SelectItem, SelectItemText,
} from 'radix-vue'

defineOptions({ inheritAttrs: false })

const props = withDefaults(defineProps<{
  modelValue?: string | number
  label?: string
  hint?: string
  error?: string
  placeholder?: string
  required?: boolean
  disabled?: boolean
  readonly?: boolean
  id?: string
  name?: string
  options: Array<{ value: string | number; label: string; disabled?: boolean }>
}>(), {
  modelValue: undefined,
  label: undefined,
  hint: undefined,
  error: undefined,
  placeholder: undefined,
  required: false,
  disabled: false,
  readonly: false,
  id: undefined,
  name: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'blur': [event: FocusEvent]
}>()

const { t } = useI18n()
// Stable ID for aria associations — random suffix avoids collisions when multiple selects are on page
const inputId = computed(() => props.id ?? `c-select-${Math.random().toString(36).slice(2, 7)}`)

// Radix SelectItem rejects empty-string values — use a sentinel internally
const EMPTY_SENTINEL = '__empty__'

// Radix works with strings; coerce numeric values to string for the SelectRoot binding
const stringValue = computed(() => {
  if (props.modelValue === undefined || props.modelValue === null) return undefined
  const s = String(props.modelValue)
  return s === '' ? EMPTY_SENTINEL : s
})

// Resolve the display label for readonly mode
const readonlyLabel = computed(() =>
  props.options.find(o => String(o.value) === String(props.modelValue))?.label ?? props.modelValue,
)

const isRequiredMissing = computed(() => {
  if (!props.required) return false
  const val = stringValue.value
  return val === undefined || val === EMPTY_SENTINEL
})

const { displayError, onFieldBlur, onFieldInput } = useFieldValidation({
  externalError: () => props.error,
  readonly: () => props.readonly,
  getInternalError: () => (isRequiredMissing.value ? t('common.required') : undefined),
  watchSources: [
    () => props.required,
    () => props.readonly,
    () => stringValue.value,
  ],
})

const nativeProxyValue = computed(() => {
  const val = stringValue.value
  if (val === undefined || val === EMPTY_SENTINEL) return ''
  return val
})

function handleUpdate(val: string) {
  emit('update:modelValue', val === EMPTY_SENTINEL ? '' : val)
  onFieldInput()
}

function handleBlur(event: FocusEvent) {
  onFieldBlur()
  emit('blur', event)
}
</script>

<template>
  <div class="flex flex-col gap-1">
    <label
      v-if="props.label"
      :for="inputId"
      class="form-label"
    >
      {{ props.label }}
      <span
        v-if="props.required"
        class="text-error ml-px"
        :aria-label="t('common.requiredAria')"
      >*</span>
    </label>

    <!-- Read-only display — mirrors existing behaviour -->
    <span
      v-if="props.readonly"
      :id="inputId"
      class="flex items-center h-[var(--control-height)] px-4 text-md text-on-surface
             bg-surface-container-low rounded-radius-md w-full box-border"
    >{{ readonlyLabel }}</span>

    <!-- Interactive select via Radix -->
    <SelectRoot
      v-else
      :model-value="stringValue"
      :disabled="props.disabled"
      @update:model-value="handleUpdate"
    >
      <SelectTrigger
        v-bind="$attrs"
        :id="inputId"
        :aria-describedby="displayError ? `${inputId}-error` : props.hint ? `${inputId}-hint` : undefined"
        :aria-invalid="!!displayError"
        :aria-required="props.required"
        class="flex items-center justify-between w-full h-[var(--control-height)] px-3 rounded-xl
               bg-surface-container text-on-surface text-md outline-none cursor-pointer
               focus:ring-2 focus:ring-primary/30 focus:bg-surface-container-low
               data-[disabled]:opacity-50 data-[disabled]:cursor-not-allowed
               transition-[background,box-shadow] duration-fast"
        :class="[
          displayError ? 'ring-2 ring-error' : '',
          !props.modelValue ? 'text-on-surface-variant' : '',
        ]"
        @blur="handleBlur"
      >
        <SelectValue :placeholder="props.placeholder ?? t('common.select')" />
        <!-- Chevron icon — inline SVG to avoid external dependency -->
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
      </SelectTrigger>

      <SelectPortal>
        <SelectContent
          position="popper"
          :side-offset="4"
          class="min-w-[var(--radix-select-trigger-width)] rounded-xl
                 bg-surface-container-lowest shadow-md z-toast overflow-hidden"
        >
          <SelectViewport class="p-1">
            <SelectItem
              v-for="opt in props.options"
              :key="opt.value"
              :value="String(opt.value) === '' ? EMPTY_SENTINEL : String(opt.value)"
              :disabled="opt.disabled"
              class="flex items-center px-3 py-2 text-md text-on-surface rounded-md
                     cursor-pointer outline-none select-none
                     hover:bg-surface-container
                     data-[highlighted]:bg-surface-container
                     data-[state=checked]:font-medium
                     data-[disabled]:opacity-40 data-[disabled]:cursor-not-allowed"
            >
              <SelectItemText>{{ opt.label }}</SelectItemText>
            </SelectItem>
          </SelectViewport>
        </SelectContent>
      </SelectPortal>
    </SelectRoot>

    <CFormFieldProxyFeedback
      :readonly="props.readonly"
      :name="props.name"
      :required="props.required"
      proxy-type="text"
      :proxy-value="nativeProxyValue"
      :display-error="displayError"
      :error-id="`${inputId}-error`"
    />
    <p
      v-if="!props.readonly && !displayError && props.hint"
      :id="`${inputId}-hint`"
      class="form-hint"
    >
      {{ props.hint }}
    </p>
  </div>
</template>

