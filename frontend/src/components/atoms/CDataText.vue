<script setup lang="ts">
type DataTextSize = 'sm' | 'md' | 'lg'
type DataTextTag = 'span' | 'strong' | 'div'

withDefaults(defineProps<{
  size?: DataTextSize
  as?: DataTextTag
}>(), {
  size: 'md',
  as: 'span',
})

const sizeClass: Record<DataTextSize, string> = {
  sm: 'text-xs',
  md: 'text-sm',
  lg: 'text-base',
}
</script>

<template>
  <component
    :is="as"
    :class="['font-mono font-medium lining-nums tabular-nums', sizeClass[size]]"
  >
    <slot></slot>
  </component>
</template>
