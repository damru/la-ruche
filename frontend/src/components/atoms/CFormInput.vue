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
  step?: number | string
  min?: number | string
  max?: number | string
  maxlength?: number | string
  autocomplete?: string
  inputmode?: 'none' | 'text' | 'tel' | 'url' | 'email' | 'numeric' | 'decimal' | 'search'
  name?: string
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
  step: undefined,
  min: undefined,
  max: undefined,
  maxlength: undefined,
  autocomplete: undefined,
  inputmode: undefined,
  name: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'blur': [event: FocusEvent]
}>()

const { t } = useI18n()
const inputId = props.id ?? `c-input-${Math.random().toString(36).slice(2, 7)}`
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

    <span
      v-if="readonly"
      :id="inputId"
      class="c-field__readonly"
    >{{ modelValue }}</span>

    <div
      v-else
      class="c-field__control"
    >
      <span
        v-if="$slots.prefix"
        class="c-field__prefix"
      >
        <slot name="prefix"></slot>
      </span>
      <input
        :id="inputId"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :required="required"
        :step="step"
        :min="min"
        :max="max"
        :maxlength="maxlength"
        :autocomplete="autocomplete"
        :inputmode="inputmode"
        :name="name"
        :aria-describedby="error ? `${inputId}-error` : hint ? `${inputId}-hint` : undefined"
        :aria-invalid="!!error"
        :class="['c-field__input', {
          'c-field__input--error': error,
          'c-field__input--with-prefix': $slots.prefix,
          'c-field__input--with-suffix': $slots.suffix,
        }]"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        @blur="emit('blur', $event)"
      />
      <span
        v-if="$slots.suffix"
        class="c-field__suffix"
      >
        <slot name="suffix"></slot>
      </span>
    </div>

    <p
      v-if="!readonly && error"
      :id="`${inputId}-error`"
      class="c-field__error"
      role="alert"
    >
      <CIconXCircle
        class="icon-2xs"
        aria-hidden="true"
      />
      {{ error }}
    </p>
    <p
      v-else-if="!readonly && hint"
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

.c-field__input {
  box-sizing: border-box;
  height: var(--control-height);
  padding: 0 var(--space-4);
  background: var(--color-surface-container);
  border: none;
  border-radius: var(--radius-xl);
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
  background: var(--color-surface-container-low);
  box-shadow: var(--shadow-primary-ring);
}

.c-field__input--error {
  box-shadow: var(--shadow-error-ring);
}

.c-field__control { position: relative; display: flex; align-items: center; }

.c-field__prefix {
  position: absolute;
  left: var(--space-3);
  display: inline-flex;
  align-items: center;
  pointer-events: none;
  color: var(--color-on-surface-variant);
}

.c-field__prefix :deep(svg) {
  width: var(--icon-sm);
  height: var(--icon-sm);
}

.c-field__input--with-prefix {
  padding-left: calc(var(--space-3) + var(--icon-sm) + var(--space-2));
}

.c-field__suffix {
  position: absolute;
  right: var(--space-2);
  display: inline-flex;
  align-items: center;
  color: var(--color-on-surface-variant);
}

.c-field__suffix :deep(svg) {
  width: var(--icon-sm);
  height: var(--icon-sm);
}

.c-field__input--with-suffix {
  padding-right: calc(var(--space-3) + var(--icon-sm) + var(--space-3));
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
  border-radius: var(--radius-xl);
  width: 100%;
  box-sizing: border-box;
}

.c-field__error {
  font-size: var(--text-xs);
  color: var(--color-error);
  display: flex;
  align-items: center;
  gap: var(--space-1);
}
</style>
