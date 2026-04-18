<script setup lang="ts">
const props = withDefaults(defineProps<{
  modelValue?: string
  label?: string
  hint?: string
  error?: string
  placeholder?: string
  required?: boolean
  disabled?: boolean
  rows?: number
  id?: string
}>(), {
  required: false,
  disabled: false,
  rows: 4,
})

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
const inputId = props.id ?? `c-textarea-${Math.random().toString(36).slice(2, 7)}`
</script>

<template>
  <div class="c-field">
    <label v-if="label" :for="inputId" class="c-field__label">
      {{ label }}
      <span v-if="required" class="c-field__required">*</span>
    </label>

    <textarea
      :id="inputId"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :required="required"
      :rows="rows"
      :aria-describedby="error ? `${inputId}-error` : hint ? `${inputId}-hint` : undefined"
      :aria-invalid="!!error"
      :class="['c-field__textarea', { 'c-field__textarea--error': error }]"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    />

    <p v-if="error" :id="`${inputId}-error`" class="c-field__error" role="alert">{{ error }}</p>
    <p v-else-if="hint" :id="`${inputId}-hint`" class="c-field__hint">{{ hint }}</p>
  </div>
</template>

<style scoped>
.c-field { display: flex; flex-direction: column; gap: var(--space-1); }
.c-field__label { font-size: 13px; font-weight: 500; color: var(--color-on-surface); }
.c-field__required { color: var(--color-error); margin-left: 2px; }

.c-field__textarea {
  padding: var(--space-3) var(--space-4);
  background: var(--color-surface-container-high);
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-body);
  font-size: 14px;
  color: var(--color-on-surface);
  outline: none;
  resize: vertical;
  min-height: 80px;
  width: 100%;
  line-height: 1.5;
  transition: background var(--transition-fast), box-shadow var(--transition-fast);
}
.c-field__textarea::placeholder { color: var(--color-on-surface-variant); }
.c-field__textarea:focus {
  background: var(--color-surface-container);
  box-shadow: 0 0 0 2px rgba(100, 63, 213, 0.35);
}
.c-field__textarea--error { box-shadow: 0 0 0 2px rgba(186, 26, 26, 0.35); }
.c-field__textarea:disabled { opacity: 0.5; cursor: not-allowed; }
.c-field__hint { font-size: 12px; color: var(--color-on-surface-variant); }
.c-field__error { font-size: 12px; color: var(--color-error); }
</style>
