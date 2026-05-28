<template>
  <div class="c-card-note">
    <slot></slot>
  </div>
</template>

<style scoped>
.c-card-note {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1) var(--space-2);
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-low);
  padding: var(--space-3);
  font-size: var(--text-xs);
  line-height: 1.4;
}

.c-card-note :deep(svg) {
  width: var(--icon-xs);
  height: var(--icon-xs);
  flex-shrink: 0;
}
</style>