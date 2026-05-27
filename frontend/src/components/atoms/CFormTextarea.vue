<script setup lang="ts">
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

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
const inputId = props.id ?? `c-textarea-${Math.random().toString(36).slice(2, 7)}`
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
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :required="required"
      :rows="rows"
      :aria-describedby="error ? `${inputId}-error` : hint ? `${inputId}-hint` : undefined"
      :aria-invalid="!!error"
      :class="['c-field__textarea', { 'c-field__textarea--error': error }]"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    ></textarea>

    <p
      v-if="!readonly && error"
      :id="`${inputId}-error`"
      class="c-field__error"
      role="alert"
    >
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

.c-field__textarea {
  padding: var(--space-3) var(--space-4);
  background: var(--color-surface-container);
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-body);
  font-size: var(--text-md);
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
  background: var(--color-surface-container-low);
  box-shadow: var(--shadow-primary-ring);
}
.c-field__textarea--error { box-shadow: var(--shadow-error-ring); }
.c-field__textarea:disabled { opacity: 0.5; cursor: not-allowed; }
.c-field__error { font-size: var(--text-xs); color: var(--color-error); }

.c-field__readonly {
  display: block;
  padding: var(--space-3) var(--space-4);
  font-family: var(--font-body);
  font-size: var(--text-md);
  color: var(--color-on-surface);
  background: var(--color-surface-container-low);
  border-radius: var(--radius-md);
  width: 100%;
  box-sizing: border-box;
  min-height: 80px;
  line-height: 1.5;
  white-space: pre-wrap;
}
</style>
