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
    class="flex flex-wrap items-center gap-x-2 gap-y-1"
  >
    <slot></slot>
  </component>
</template>

<style scoped>
.c-meta-line {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-1) var(--space-2);
}
</style>
