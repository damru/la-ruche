<template>
  <div class="c-topbar">
    <div class="c-topbar__left">
      <slot name="left"></slot>
    </div>
    <div class="c-topbar__right">
      <slot name="right"></slot>
    </div>
  </div>
</template>

<style scoped>
.c-topbar {
  position: sticky;
  top: 0;
  z-index: var(--z-nav);
  min-height: var(--topbar-height);
  padding: var(--space-3) var(--space-10);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  background: color-mix(in oklch, var(--color-surface) 88%, transparent);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  box-shadow: var(--shadow-md);
  transition: background var(--transition-base);
}

.c-topbar__left {
  flex: 1 1 auto;
  min-width: 0;
}

.c-topbar__right {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

/* Greeting helper */
.c-topbar :deep(.topbar-greeting-title) {
  font-family: var(--font-display);
  font-size: 15px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 20;
  letter-spacing: -0.01em;
  color: var(--color-on-surface);
}

.c-topbar :deep(.topbar-greeting-date) {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
}

/* ── Mobile ──────────────────────────────────────────────────────── */
@media (max-width: 640px) {
  .c-topbar {
    padding: 0 var(--space-5);
  }
}
</style>
