<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { CheckboxRoot, CheckboxIndicator } from 'radix-vue'
import { useFieldValidation } from '@/composables/useFieldValidation'

const props = withDefaults(defineProps<{
  /**
   * Boolean mode: bind a boolean ref.
   * Array mode: bind an array ref and set :value to the item this checkbox represents.
   */
  modelValue?: boolean | unknown[]
  /** Item value for array-mode v-model. */
  value?: unknown
  /** Optional text label. Use the default slot for rich/conditional content instead. */
  label?: string
  disabled?: boolean
  error?: string
  required?: boolean
  name?: string
}>(), {
  modelValue: undefined,
  value: undefined,
  label: undefined,
  disabled: false,
  error: undefined,
  required: false,
  name: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean | unknown[]]
}>()

const { t } = useI18n()

const isChecked = computed<boolean>(() => {
  if (Array.isArray(props.modelValue)) return props.modelValue.includes(props.value)
  return !!props.modelValue
})

const { displayError, onFieldBlur, onFieldInput } = useFieldValidation({
  externalError: () => props.error,
  getInternalError: () => {
    if (!props.required) return undefined
    return isChecked.value ? undefined : t('common.required')
  },
  watchSources: [
    () => props.required,
    () => isChecked.value,
  ],
})

function handleCheckedChange(checked: boolean) {
  onFieldBlur()
  if (Array.isArray(props.modelValue)) {
    const arr = [...props.modelValue]
    if (checked) {
      arr.push(props.value)
    } else {
      const i = arr.indexOf(props.value)
      if (i > -1) arr.splice(i, 1)
    }
    emit('update:modelValue', arr)
  } else {
    emit('update:modelValue', checked)
  }
  onFieldInput()
}

function handleBlur() {
  onFieldBlur()
}

defineOptions({ inheritAttrs: false })
</script>

<template>
  <div class="flex flex-col gap-1">
    <label
      class="flex items-center gap-2 cursor-pointer select-none"
      :class="props.disabled ? 'opacity-50 cursor-not-allowed' : ''"
    >
      <CheckboxRoot
        v-bind="$attrs"
        :checked="isChecked"
        :disabled="props.disabled"
        :aria-required="props.required"
        :aria-invalid="!!displayError"
        class="min-w-[unset]! min-h-[unset]! size-4! rounded-full border border-outline-variant bg-surface-container-low
               data-[state=checked]:bg-primary data-[state=checked]:border-primary
               flex items-center justify-center outline-none flex-shrink-0
               focus-visible:ring-2 focus-visible:ring-primary/30"
        @blur="handleBlur"
        @update:checked="handleCheckedChange"
      >
        <CheckboxIndicator class="text-on-primary">
          <svg
            viewBox="0 0 16 16"
            class="size-3"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <path
              d="M3 8l3.5 3.5L13 4.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </CheckboxIndicator>
      </CheckboxRoot>
      <span
        v-if="props.label"
        class="text-sm text-on-surface"
      >
        {{ props.label }}
        <span
          v-if="props.required"
          class="ml-px text-error"
          :aria-label="t('common.requiredAria')"
        >*</span>
      </span>
      <slot v-else></slot>
    </label>
    <CFormFieldProxyFeedback
      :name="props.name"
      :required="props.required"
      proxy-type="checkbox"
      :proxy-checked="isChecked"
      :display-error="displayError"
    />
  </div>
</template>
