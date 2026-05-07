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
    class="c-card-meta"
  >
    <slot></slot>
  </component>
</template>

<style scoped>
.c-card-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-1) var(--space-2);
  font-size: var(--text-2xs);
  line-height: 1.4;
}

.c-card-meta :deep(svg) {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}
</style>
