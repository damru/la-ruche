<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFieldValidation } from '@/composables/useFieldValidation'

const props = withDefaults(defineProps<{
  modelValue?: string
  label?: string
  hint?: string
  error?: string
  placeholder?: string
  required?: boolean
  disabled?: boolean
  readonly?: boolean
  rows?: number
  id?: string
}>(), {
  modelValue: undefined,
  label: undefined,
  hint: undefined,
  error: undefined,
  placeholder: undefined,
  required: false,
  disabled: false,
  readonly: false,
  rows: 4,
  id: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'blur': [event: FocusEvent]
}>()

const { t } = useI18n()
const inputId = props.id ?? `c-textarea-${Math.random().toString(36).slice(2, 7)}`
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const { displayError, onFieldBlur, onFieldInput } = useFieldValidation({
  externalError: () => props.error,
  readonly: () => props.readonly,
  getInternalError: () => {
    const textarea = textareaRef.value
    if (!textarea || textarea.validity.valid) return undefined
    return textarea.validity.valueMissing ? t('common.required') : textarea.validationMessage
  },
  watchSources: [
    () => props.modelValue,
    () => props.required,
    () => props.readonly,
  ],
})

function handleInput(event: Event) {
  emit('update:modelValue', (event.target as HTMLTextAreaElement).value)
  onFieldInput()
}

function handleBlur(event: FocusEvent) {
  onFieldBlur()
  emit('blur', event)
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
      >*</span>
    </label>

    <span
      v-if="readonly"
      :id="inputId"
      class="c-field__readonly"
    >{{ modelValue }}</span>

    <textarea
      v-else
      :id="inputId"
      ref="textareaRef"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :required="required"
      :rows="rows"
      :aria-describedby="displayError ? `${inputId}-error` : hint ? `${inputId}-hint` : undefined"
      :aria-invalid="!!displayError"
      :class="['c-field__textarea', { 'c-field__textarea--error': displayError }]"
      @input="handleInput"
      @blur="handleBlur"
    ></textarea>

    <CFormFieldProxyFeedback
      :readonly="readonly"
      :show-proxy="false"
      :display-error="displayError"
      :error-id="`${inputId}-error`"
      :hint="hint"
      :hint-id="`${inputId}-hint`"
      hint-class="form-hint"
    />
  </div>
</template>

<style scoped>
.c-field { display: flex; flex-direction: column; gap: var(--space-1); }
.c-field__required { color: var(--color-error); margin-left: 2px; }

.c-field__textarea {
  padding: var(--space-3) var(--space-4);
  background: var(--color-surface-container);
  border: none;
  border-radius: var(--radius-xl);
  font-family: var(--font-body);
  font-size: var(--text-md);
  color: var(--color-on-surface);
  outline: none;
  resize: vertical;
  min-height: 80px;
  width: 100%;
  line-height: var(--leading-normal);
  transition: background var(--transition-fast), box-shadow var(--transition-fast);
}
.c-field__textarea::placeholder { color: var(--color-on-surface-variant); }
.c-field__textarea:focus {
  background: var(--color-surface-container-low);
  box-shadow: var(--shadow-primary-ring);
}
.c-field__textarea--error { box-shadow: var(--shadow-error-ring); }
.c-field__textarea:disabled { opacity: 0.5; cursor: not-allowed; }

.c-field__readonly {
  display: block;
  padding: var(--space-3) var(--space-4);
  font-family: var(--font-body);
  font-size: var(--text-md);
  color: var(--color-on-surface);
  background: var(--color-surface-container-low);
  border-radius: var(--radius-xl);
  width: 100%;
  box-sizing: border-box;
  min-height: 80px;
  line-height: var(--leading-normal);
  white-space: pre-wrap;
}
</style>
