<template>
  <div class="c-card-note">
    <slot></slot>
  </div>
</template>

<style scoped>
.c-card-note {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  border-radius: var(--radius-md);
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
  padding: var(--space-3) var(--space-4);
  font-size: var(--text-xs);
  line-height: 1.4;
}

.c-card-note :deep(svg) {
  width: var(--icon-xs);
  height: var(--icon-xs);
  flex-shrink: 0;
}
</style>