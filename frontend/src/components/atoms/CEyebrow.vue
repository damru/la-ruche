<script setup lang="ts">
const props = defineProps<{
  as?: 'p' | 'span' | 'div'
}>()

const tag = props.as || 'p'
</script>

<template>
  <component :is="tag" class="page-eyebrow">
    <slot />
  </component>
</template>

<style scoped>
.page-eyebrow {
  font-size: var(--text-2xs);
  font-weight: var(--fw-semibold);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-primary);
}
</style>