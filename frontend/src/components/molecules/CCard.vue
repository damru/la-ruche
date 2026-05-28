<script setup lang="ts">
// Molecule component — CCard
// Surface container-lowest, shadow ambient
// "No-Line" rule: no internal borders
// level 1 (outermost) → padding lg / radius xl
// level 2 (default)   → padding md / radius lg
// level 3             → padding sm / radius md
// level 4 (innermost) → padding xs / radius sm

withDefaults(defineProps<{
  level?: 1 | 2 | 3 | 4
  shadow?: boolean
  interactive?: boolean   // hover lift
  gap?: 'none' | 'sm' | 'md'
}>(), {
  level: 2,
  shadow: true,
  interactive: false,
  gap: 'md',
})
</script>

<template>
  <div
    :class="[
      'c-card',
      `c-card--level-${level}`,
      `c-card--gap-${gap}`,
      { 'c-card--shadow': shadow, 'c-card--interactive': interactive }
    ]"
  >
    <slot></slot>
  </div>
</template>

<style scoped>
.c-card {
  background: var(--tone-bg, var(--color-surface-container-lowest));
  overflow-x: auto;
  min-width: 0;
  height: fit-content;
  color: var(--tone-fg, var(--color-on-surface-variant));
  display: flex;
  flex-direction: column;
  flex: 1;
}

/* level 1 — outermost: padding lg (24px) + radius xl (24px) */
.c-card--level-1 { padding: var(--space-6); border-radius: var(--radius-xl); }
/* level 2 — default: padding md (20px) + radius lg (16px) */
.c-card--level-2 { padding: var(--space-5); border-radius: var(--radius-lg); }
/* level 3 — nested: padding sm (16px) + radius md (12px) */
.c-card--level-3 { padding: var(--space-4); border-radius: var(--radius-md); }
/* level 4 — innermost: padding xs (12px) + radius sm (8px) */
.c-card--level-4 { padding: var(--space-3); border-radius: var(--radius-sm); }

.c-card--no-padding { padding: 0; }

.c-card--gap-none { gap: 0; }
.c-card--gap-sm   { gap: var(--space-3); }
.c-card--gap-md   { gap: var(--space-4); }

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

</style>
