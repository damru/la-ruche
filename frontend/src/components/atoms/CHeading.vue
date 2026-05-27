<script setup lang="ts">
export type HeadingSize = 'lg' | 'md' | 'sm'
export type HeadingAs = 'h1' | 'h2' | 'h3' | 'p'

withDefaults(defineProps<{
  size?: HeadingSize
  as?: HeadingAs
}>(), {
  size: 'md',
  as: 'h2',
})

const sizeClass: Record<HeadingSize, string> = {
  lg: 'text-2xl',
  md: 'text-xl',
  sm: 'text-heading-sm',
}
</script>

<template>
  <component
    :is="$props.as"
    class="font-serif font-semibold italic leading-tight tracking-tight text-on-surface"
    :class="sizeClass[$props.size ?? 'md']"
  ><slot /></component>
</template>
