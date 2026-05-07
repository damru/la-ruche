<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const props = withDefaults(defineProps<{
  modelValue?: string | number
  label?: string
  hint?: string
  error?: string
  placeholder?: string
  type?: string
  required?: boolean
  disabled?: boolean
  readonly?: boolean
  id?: string
}>(), {
  modelValue: undefined,
  label: undefined,
  hint: undefined,
  error: undefined,
  placeholder: undefined,
  type: 'text',
  required: false,
  disabled: false,
  readonly: false,
  id: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const { t } = useI18n()
const inputId = props.id ?? `c-input-${Math.random().toString(36).slice(2, 7)}`
</script>

<template>
  <div class="c-field">
    <label
      v-if="label"
      :for="inputId"
      class="c-field__label"
    >
      {{ label }}
      <span
        v-if="required"
        class="c-field__required"
        :aria-label="t('common.requiredAria')"
      >*</span>
    </label>

    <span
      v-if="readonly"
      :id="inputId"
      class="c-field__readonly"
    >{{ modelValue }}</span>

    <input
      v-else
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

    <p
      v-if="!readonly && error"
      :id="`${inputId}-error`"
      class="c-field__error"
      role="alert"
    >
      <CIconXCircle
        width="12"
        height="12"
        aria-hidden="true"
      />
      {{ error }}
    </p>
    <p
      v-else-if="!readonly && hint"
      :id="`${inputId}-hint`"
      class="c-field__hint"
    >
      {{ hint }}
    </p>
  </div>
</template>

<style scoped>
.c-field { display: flex; flex-direction: column; gap: var(--space-1); }

.c-field__label {
  font-family: var(--font-body);
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
  color: var(--color-on-surface);
}

.c-field__required { color: var(--color-error); margin-left: 2px; }

.c-field__input {
  box-sizing: border-box;
  height: var(--control-height);
  padding: 0 var(--space-4);
  background: var(--color-surface-container-high);
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-body);
  font-size: var(--text-md);
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
  box-shadow: var(--shadow-primary-ring);
}

.c-field__input--error {
  box-shadow: var(--shadow-error-ring);
}

.c-field__input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.c-field__readonly {
  display: flex;
  align-items: center;
  height: var(--control-height);
  padding: 0 var(--space-4);
  font-family: var(--font-body);
  font-size: var(--text-md);
  color: var(--color-on-surface);
  background: var(--color-surface-container-low);
  border-radius: var(--radius-md);
  width: 100%;
  box-sizing: border-box;
}

.c-field__hint {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
}

.c-field__error {
  font-size: var(--text-xs);
  color: var(--color-error);
  display: flex;
  align-items: center;
  gap: var(--space-1);
}
</style>
