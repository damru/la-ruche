<script setup lang="ts">
// Molecule component — CCard
// Surface container-lowest, shadow ambient, radius-lg
// "No-Line" rule: no internal borders

withDefaults(defineProps<{
  padding?: 'sm' | 'md' | 'lg' | 'none'
  shadow?: boolean
  interactive?: boolean   // hover lift
}>(), {
  padding: 'md',
  shadow: true,
  interactive: false,
})
</script>

<template>
  <div
    :class="[
      'c-card',
      `c-card--pad-${padding}`,
      { 'c-card--shadow': shadow, 'c-card--interactive': interactive }
    ]"
  >
    <slot></slot>
  </div>
</template>

<style scoped>
.c-card {
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  overflow-x: auto;
  min-width: 0;
  color: var(--color-on-surface-variant);
}

.c-card--pad-none { padding: 0; }
.c-card--pad-sm   { padding: var(--space-4); }
.c-card--pad-md   { padding: var(--space-5); }
.c-card--pad-lg   { padding: var(--space-6); }

.c-card--shadow { box-shadow: var(--shadow-md); }

.c-card--interactive {
  cursor: pointer;
  transition: transform var(--transition-fast), box-shadow var(--transition-fast);
}
.c-card--shadow.c-card--interactive:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-lg);
}
.c-card--interactive:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}
.c-card--interactive:active { transform: translateY(0); }
.c-card--interactive:hover :deep(h2), .c-card--interactive:hover :deep(h3) {
  color: var(--color-primary);
}

</style>
