<script setup lang="ts">
// Composant atomique — CSelect
// Same aesthetic as CInput — "sunken pit"

const props = withDefaults(defineProps<{
  modelValue?: string | number
  label?: string
  hint?: string
  error?: string
  placeholder?: string
  required?: boolean
  disabled?: boolean
  id?: string
  options: Array<{ value: string | number; label: string; disabled?: boolean }>
}>(), {
  required: false,
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const inputId = props.id ?? `c-select-${Math.random().toString(36).slice(2, 7)}`
</script>

<template>
  <div class="c-field">
    <label v-if="label" :for="inputId" class="c-field__label">
      {{ label }}
      <span v-if="required" class="c-field__required" aria-label="obligatoire">*</span>
    </label>

    <div class="c-field__select-wrap">
      <select
        :id="inputId"
        :value="modelValue"
        :disabled="disabled"
        :required="required"
        :aria-describedby="error ? `${inputId}-error` : hint ? `${inputId}-hint` : undefined"
        :aria-invalid="!!error"
        :class="['c-field__input', { 'c-field__input--error': error, 'c-field__input--placeholder': !modelValue }]"
        @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
      >
        <option v-if="placeholder" value="" disabled :selected="!modelValue">{{ placeholder }}</option>
        <option
          v-for="opt in options"
          :key="opt.value"
          :value="opt.value"
          :disabled="opt.disabled"
        >{{ opt.label }}</option>
      </select>
      <svg class="c-field__chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <path d="M6 9l6 6 6-6"/>
      </svg>
    </div>

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
  font-size: 13px;
  font-weight: 500;
  color: var(--color-on-surface);
}

.c-field__required { color: var(--color-error); margin-left: 2px; }

.c-field__select-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.c-field__input {
  appearance: none;
  width: 100%;
  height: 44px;
  padding: 0 var(--space-10) 0 var(--space-4);
  background: var(--color-surface-container-high);
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-body);
  font-size: 14px;
  color: var(--color-on-surface);
  outline: none;
  cursor: pointer;
  transition:
    background var(--transition-fast),
    box-shadow var(--transition-fast);
}

.c-field__input--placeholder { color: var(--color-on-surface-variant); }

.c-field__input:focus {
  background: var(--color-surface-container);
  box-shadow: 0 0 0 2px rgba(100, 63, 213, 0.35);
}

.c-field__input--error { box-shadow: 0 0 0 2px rgba(186, 26, 26, 0.35); }

.c-field__input:disabled { opacity: 0.5; cursor: not-allowed; }

.c-field__chevron {
  position: absolute;
  right: var(--space-3);
  width: 16px;
  height: 16px;
  color: var(--color-on-surface-variant);
  pointer-events: none;
  flex-shrink: 0;
}

.c-field__hint { font-size: 12px; color: var(--color-on-surface-variant); }

.c-field__error {
  font-size: 12px;
  color: var(--color-error);
  display: flex;
  align-items: center;
  gap: var(--space-1);
}
</style>
