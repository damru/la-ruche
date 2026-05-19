<script setup lang="ts">
import { Separator } from 'radix-vue';

/** Separator atom.
 *  - horizontal (default): small circular dot — use between inline/meta items
 *  - vertical: full-width rule — use between stacked sections
 */
withDefaults(defineProps<{
  variant?: 'horizontal' | 'vertical'
}>(), {
  variant: 'horizontal',
})
</script>

<template>
  <Separator
    :class="{
      'c-separator--horizontal': variant === 'horizontal',
      'c-separator--vertical': variant === 'vertical',
    }"
    :decorative="true"
    :orientation="variant"
  />
</template>

<style scoped>
/* Horizontal — circular dot between inline items */
.c-separator--horizontal {
  width: 0.25em;
  height: 0.25em;
  border-radius: var(--radius-full);
  background: var(--color-outline);
}

/* Vertical — thin rule between stacked sections */
.c-separator--vertical {
  height: 1px;
  background: var(--color-outline-light);
}
</style>
