<script setup lang="ts">
const props = withDefaults(defineProps<{
  modelValue: number | string
  label: string
  type?: 'number' | 'text'
  step?: number | string
  min?: number | string
  max?: number | string
  placeholder?: string
  disabled?: boolean
  readonly?: boolean
  id?: string
}>(), {
  type: 'number',
  step: undefined,
  min: undefined,
  max: undefined,
  placeholder: undefined,
  disabled: false,
  readonly: false,
  id: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: number | string]
}>()

const inputId = props.id ?? `c-chip-input-${Math.random().toString(36).slice(2, 7)}`

function onInput(raw: string) {
  const cast = props.type === 'number' ? (raw === '' ? 0 : Number(raw)) : raw
  emit('update:modelValue', cast)
}
</script>

<template>
  <span class="c-chip">
    <span class="c-chip__label">{{ label }}</span>
    <span
      v-if="readonly"
      class="c-chip__value c-chip__value--readonly"
    >{{ modelValue }}</span>
    <input
      v-else
      :id="inputId"
      :type="type"
      :value="modelValue"
      :step="step"
      :min="min"
      :max="max"
      :placeholder="placeholder"
      :disabled="disabled"
      class="c-chip__value"
      @input="onInput(($event.target as HTMLInputElement).value)"
    />
  </span>
</template>

<style scoped>
.c-chip {
  display: inline-flex;
  align-items: stretch;
  background: var(--color-secondary-container);
  border-radius: 9999px;
  overflow: clip;
  cursor: default;
}

.c-chip__label {
  padding: 5px 8px 5px 12px;
  font-size: var(--text-xs);
  font-weight: var(--fw-semibold);
  color: var(--color-on-surface-variant);
  white-space: nowrap;
  display: flex;
  align-items: center;
}

.c-chip__value {
  box-sizing: border-box;
  width: 60px;
  padding: 5px 10px 5px 6px;
  font-family: var(--font-body);
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
  text-align: right;
  background: var(--color-surface-container);
  border: none;
  border-radius: 0 9999px 9999px 0;
  color: var(--color-on-surface);
  outline: none;
  transition: background var(--transition-fast), box-shadow var(--transition-fast);
}

.c-chip__value:focus {
  background: var(--color-surface-container-low);
  box-shadow: inset var(--shadow-primary-ring);
}

.c-chip__value:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.c-chip__value--readonly {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  cursor: default;
}
</style>
