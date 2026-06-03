<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useFieldValidation } from '@/composables/useFieldValidation'

const props = withDefaults(defineProps<{
  modelValue: string[]
  options: Array<{ value: string; label: string; disabled?: boolean }>
  label?: string
  hint?: string
  error?: string
  required?: boolean
  disabled?: boolean
  id?: string
  name?: string
}>(), {
  label: undefined,
  hint: undefined,
  error: undefined,
  required: false,
  disabled: false,
  id: undefined,
  name: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const { t } = useI18n()
const inputId = props.id ?? `c-chip-select-${Math.random().toString(36).slice(2, 7)}`
const { displayError, onFieldBlur, onFieldInput } = useFieldValidation({
  externalError: () => props.error,
  getInternalError: () => {
    if (!props.required) return undefined
    return props.modelValue.length > 0 ? undefined : t('common.required')
  },
  watchSources: [
    () => props.required,
    () => props.modelValue.length,
  ],
})

function isSelected(value: string): boolean {
  return props.modelValue.includes(value)
}

function toggle(value: string, optionDisabled?: boolean) {
  if (props.disabled || optionDisabled) return
  onFieldBlur()
  const next = isSelected(value)
    ? props.modelValue.filter(v => v !== value)
    : [...props.modelValue, value]
  emit('update:modelValue', next)
  onFieldInput()
}

function handleFocusOut(event: FocusEvent) {
  const current = event.currentTarget as HTMLElement | null
  const related = event.relatedTarget as Node | null
  if (current && related && current.contains(related)) return
  onFieldBlur()
}
</script>

<template>
  <div class="c-field">
    <label
      v-if="label"
      :for="inputId"
      class="form-label"
    >
      {{ label }}
      <span
        v-if="required"
        class="c-field__required"
        :aria-label="t('common.requiredAria')"
      >*</span>
    </label>

    <div
      :id="inputId"
      class="c-chip-select"
      role="group"
      :aria-describedby="displayError ? `${inputId}-error` : hint ? `${inputId}-hint` : undefined"
      :aria-invalid="!!displayError"
      :aria-required="required"
      @focusout="handleFocusOut"
    >
      <button
        v-for="option in options"
        :key="option.value"
        type="button"
        class="c-chip-select__chip"
        :class="{
          'c-chip-select__chip--selected': isSelected(option.value),
          'c-chip-select__chip--disabled': disabled || option.disabled,
        }"
        :disabled="disabled || option.disabled"
        :aria-pressed="isSelected(option.value)"
        @click="toggle(option.value, option.disabled)"
      >
        <CIconCheck
          v-if="isSelected(option.value)"
          class="c-chip-select__icon"
          aria-hidden="true"
        />
        {{ option.label }}
      </button>
    </div>

    <CFormFieldProxyFeedback
      :name="name"
      :required="required"
      proxy-type="text"
      :proxy-value="modelValue.join(',')"
      :display-error="displayError"
      :error-id="`${inputId}-error`"
    />
    <p
      v-if="!displayError && hint"
      :id="`${inputId}-hint`"
      class="form-hint"
    >
      {{ hint }}
    </p>
  </div>
</template>

<style scoped>
.c-field { display: flex; flex-direction: column; gap: var(--space-1); }

.c-field__required { color: var(--color-error); margin-left: 2px; }

.c-chip-select {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.c-chip-select__chip {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  height: 32px;
  min-height: unset;
  padding: 0 var(--space-3);
  border-radius: var(--radius-full);
  background: var(--color-surface-container);
  color: var(--color-on-surface);
  font-family: var(--font-body);
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
  border: 1px solid transparent;
  cursor: pointer;
  transition:
    background var(--transition-fast),
    color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.c-chip-select__chip:hover:not(:disabled) {
  background: var(--color-surface-container-low);
}

.c-chip-select__chip--selected {
  background: var(--gradient-primary, var(--color-primary));
  color: var(--color-on-primary);
}

.c-chip-select__chip--selected:hover:not(:disabled) {
  background: var(--gradient-primary, var(--color-primary));
  filter: brightness(1.05);
}

.c-chip-select__chip:disabled,
.c-chip-select__chip--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.c-chip-select__icon {
  width: var(--icon-xs);
  height: var(--icon-xs);
}

</style>
