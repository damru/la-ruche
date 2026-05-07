<script setup lang="ts">
withDefaults(defineProps<{
  as?: 'div' | 'span'
}>(), {
  as: 'div',
})
</script>

<template>
  <component
    :is="as"
    class="c-meta"
  >
    <slot></slot>
  </component>
</template>

<style scoped>
.c-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-1) var(--space-2);
}
</style>
