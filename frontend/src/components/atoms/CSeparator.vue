<script setup lang="ts">
/** Separator atom — small circular dot divider */

const props = defineProps<{
  /** Semantic HTML element */
  as?: 'span' | 'div'
}>()

const tag = props.as || 'span'
</script>

<template>
  <component :is="tag" class="c-separator" />
</template>

<style scoped>
.c-separator {
  width: 0.25em;
  height: 0.25em;
  border-radius: var(--radius-full);
  background: var(--color-outline);
  display: inline-block;
}
</style>
