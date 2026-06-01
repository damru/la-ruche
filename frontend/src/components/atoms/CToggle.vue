<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from 'radix-vue'

const props = defineProps<{
  modelValue: boolean
  disabled?: boolean
  ariaLabel?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()
</script>

<template>
  <SwitchRoot
    :checked="props.modelValue"
    :disabled="props.disabled"
    :aria-label="props.ariaLabel"
    class="relative inline-flex w-[44px]! min-w-[unset]! h-6! min-h-[unset]! rounded-full border-none p-0 cursor-pointer shrink-0 self-center
           bg-[var(--color-surface-container-high)]
           data-[state=checked]:bg-[var(--color-primary)]
           data-[disabled]:opacity-50 data-[disabled]:cursor-not-allowed
           focus-visible:outline-2 focus-visible:outline-[var(--color-primary)] focus-visible:outline-offset-2
           transition-colors"
    @update:checked="emit('update:modelValue', $event)"
  >
    <SwitchThumb
      class="block absolute top-[3px] left-[3px] size-[18px] rounded-full bg-white transition-transform
             data-[state=checked]:translate-x-5"
    />
  </SwitchRoot>
</template>
