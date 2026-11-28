<script setup lang="ts">
export type HeadingType = 'header' | 'section' | 'card'
export type HeadingVariant = 'heading' | 'title'

const props = withDefaults(defineProps<{
  type?: HeadingType
  variant?: HeadingVariant
}>(), {
  type: 'section',
  variant: 'heading',
})

const tagMap: Record<HeadingType, string> = {
  header: 'h1',
  section: 'h2',
  card: 'h3',
}

const sizeClass: Record<HeadingType, string> = {
  header: 'text-2xl',
  section: 'text-xl',
  card: 'text-lg',
}

const variantClass: Record<HeadingVariant, string> = {
  heading: 'font-serif font-semibold leading-tight tracking-tight text-on-surface',
  title: 'font-body font-semibold leading-tight text-on-surface',
}
</script>

<template>
  <component
    :is="tagMap[props.type]"
    :class="[sizeClass[props.type], variantClass[props.variant]]"
  >
    <slot></slot>
  </component>
</template>
