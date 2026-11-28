<script setup lang="ts">
type DataTextSize = '2xs' | 'xs' | 'sm' | 'md' | 'lg'
type DataTextTag = 'span' | 'strong' | 'div'

withDefaults(defineProps<{
  size?: DataTextSize
  as?: DataTextTag
}>(), {
  size: 'sm',
  as: 'span',
})

const sizeClass: Record<DataTextSize, string> = {
  '2xs': 'text-2xs',
  xs: 'text-xs',
  sm: 'text-sm',
  md: 'text-md',
  lg: 'text-lg',
}
</script>

<template>
  <component
    :is="as"
    :class="[
      'font-mono lining-nums tabular-nums',
      as === 'strong' ? 'font-semibold' : 'font-medium',
      sizeClass[size],
    ]"
  >
    <slot></slot>
  </component>
</template>
