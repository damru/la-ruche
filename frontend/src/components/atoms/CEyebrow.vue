<script setup lang="ts">
withDefaults(defineProps<{
  as?: 'p' | 'span' | 'div'
}>(), {
  as: 'p',
})

</script>

<template>
  <component
    :is="as"
    class="c-eyebrow"
  >
    <slot></slot>
  </component>
</template>

<style scoped>
.c-eyebrow {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-2xs);
  font-weight: var(--fw-semibold);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-primary);
}
</style>