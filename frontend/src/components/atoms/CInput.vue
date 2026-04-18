<script setup lang="ts">
// Composant atomique — CInput
// "Sunken pit in the page" — surface-container-high background, no border

const props = withDefaults(defineProps<{
  modelValue?: string | number
  label?: string
  hint?: string
  error?: string
  placeholder?: string
  type?: string
  required?: boolean
  disabled?: boolean
  id?: string
}>(), {
  type: 'text',
  required: false,
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const inputId = props.id ?? `c-input-${Math.random().toString(36).slice(2, 7)}`
</script>

<template>
  <div class="c-field">
    <label v-if="label" :for="inputId" class="c-field__label">
      {{ label }}
      <span v-if="required" class="c-field__required" aria-label="obligatoire">*</span>
    </label>

    <input
      :id="inputId"
      :type="type"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :required="required"
      :aria-describedby="error ? `${inputId}-error` : hint ? `${inputId}-hint` : undefined"
      :aria-invalid="!!error"
      :class="['c-field__input', { 'c-field__input--error': error }]"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />

    <p v-if="error" :id="`${inputId}-error`" class="c-field__error" role="alert">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
        <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      {{ error }}
    </p>
    <p v-else-if="hint" :id="`${inputId}-hint`" class="c-field__hint">{{ hint }}</p>
  </div>
</template>

<style scoped>
.c-field { display: flex; flex-direction: column; gap: var(--space-1); }

.c-field__label {
  font-family: var(--font-body);
  font-size: 13px;
  font-weight: 500;
  color: var(--color-on-surface);
}

.c-field__required { color: var(--color-error); margin-left: 2px; }

.c-field__input {
  height: 44px;
  padding: 0 var(--space-4);
  background: var(--color-surface-container-high);
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-body);
  font-size: 14px;
  color: var(--color-on-surface);
  outline: none;
  transition:
    background var(--transition-fast),
    box-shadow var(--transition-fast);
  width: 100%;
}

.c-field__input::placeholder { color: var(--color-on-surface-variant); }

.c-field__input:focus {
  background: var(--color-surface-container);
  box-shadow: 0 0 0 2px rgba(100, 63, 213, 0.35);
}

.c-field__input--error {
  box-shadow: 0 0 0 2px rgba(186, 26, 26, 0.35);
}

.c-field__input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.c-field__hint {
  font-size: 12px;
  color: var(--color-on-surface-variant);
}

.c-field__error {
  font-size: 12px;
  color: var(--color-error);
  display: flex;
  align-items: center;
  gap: var(--space-1);
}
</style>
