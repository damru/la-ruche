<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const props = withDefaults(defineProps<{
  modelValue: string[]
  options: Array<{ value: string; label: string; disabled?: boolean }>
  label?: string
  hint?: string
  error?: string
  required?: boolean
  disabled?: boolean
  id?: string
}>(), {
  label: undefined,
  hint: undefined,
  error: undefined,
  required: false,
  disabled: false,
  id: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const { t } = useI18n()
const inputId = props.id ?? `c-chip-select-${Math.random().toString(36).slice(2, 7)}`

function isSelected(value: string): boolean {
  return props.modelValue.includes(value)
}

function toggle(value: string, optionDisabled?: boolean) {
  if (props.disabled || optionDisabled) return
  const next = isSelected(value)
    ? props.modelValue.filter(v => v !== value)
    : [...props.modelValue, value]
  emit('update:modelValue', next)
}
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

    <div
      :id="inputId"
      class="c-chip-select"
      role="group"
      :aria-describedby="error ? `${inputId}-error` : hint ? `${inputId}-hint` : undefined"
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

    <p
      v-if="error"
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
      v-else-if="hint"
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
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
  color: var(--color-on-surface);
}

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
  width: 14px;
  height: 14px;
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
