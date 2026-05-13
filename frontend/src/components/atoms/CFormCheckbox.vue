<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  /**
   * Boolean mode: bind a boolean ref.
   * Array mode: bind an array ref and set :value to the item this checkbox represents.
   */
  modelValue?: boolean | unknown[]
  /** Item value for array-mode v-model. */
  value?: unknown
  /** Optional text label. Use the default slot for rich/conditional content instead. */
  label?: string
  disabled?: boolean
  name?: string
}>(), {
  modelValue: undefined,
  value: undefined,
  label: undefined,
  disabled: false,
  name: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean | unknown[]]
}>()

const isChecked = computed(() => {
  if (Array.isArray(props.modelValue)) return props.modelValue.includes(props.value)
  return !!props.modelValue
})

function handleChange(event: Event) {
  const checked = (event.target as HTMLInputElement).checked
  if (Array.isArray(props.modelValue)) {
    const arr = [...props.modelValue]
    if (checked) {
      arr.push(props.value)
    } else {
      const i = arr.indexOf(props.value)
      if (i > -1) arr.splice(i, 1)
    }
    emit('update:modelValue', arr)
  } else {
    emit('update:modelValue', checked)
  }
}
</script>

<template>
  <label class="checkbox-label">
    <input
      type="checkbox"
      :checked="isChecked"
      :disabled="disabled"
      :name="name"
      @change="handleChange"
    />
    <span v-if="label">{{ label }}</span>
    <slot v-else />
  </label>
</template>
