<script setup lang="ts">
// Molecule component — CCard
// Surface container-lowest, shadow ambient, radius-lg
// "No-Line" rule: no internal borders

withDefaults(defineProps<{
  padding?: 'xs' | 'sm' | 'md' | 'lg' | 'none'
  radius?: 'xs' | 'sm' | 'md' | 'lg' | 'none'
  shadow?: boolean
  interactive?: boolean   // hover lift
}>(), {
  padding: 'md',
  radius: 'lg',
  shadow: true,
  interactive: false,
})
</script>

<template>
  <div
    :class="[
      'c-card',
      `c-card--pad-${padding}`,
      `c-card--radius-${radius}`,
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
}

.c-card--radius-none { border-radius: 0; }
.c-card--radius-xs   { border-radius: var(--radius-xs, var(--radius-sm)); }
.c-card--radius-sm   { border-radius: var(--radius-sm); }
.c-card--radius-md   { border-radius: var(--radius-md); }
.c-card--radius-lg   { border-radius: var(--radius-lg); }

.c-card--pad-none { padding: 0; }
.c-card--pad-xs   { padding: var(--space-3); }
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

</style>
