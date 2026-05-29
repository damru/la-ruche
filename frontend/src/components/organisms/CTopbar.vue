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
  height: var(--topbar-height);
  padding: 0 var(--space-10);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  background: rgba(248, 249, 250, 0.88);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  box-shadow: 0 10px 24px rgba(25, 28, 29, 0.05);
  transition: background var(--transition-base);
}

[data-theme="dark"] .c-topbar {
  background: rgba(17, 19, 24, 0.88);
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

.c-topbar :deep(.topbar-breadcrumb) {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
  color: var(--color-on-surface-variant);
}

.c-topbar :deep(.topbar-breadcrumb span) {
  min-width: 0;
  overflow: clip;
  text-overflow: ellipsis;
}

.c-topbar :deep(.topbar-breadcrumb svg) {
  flex: 0 0 auto;
  width: var(--icon-xs);
  height: var(--icon-xs);
  color: var(--color-outline-variant);
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

.c-topbar :deep(.topbar-breadcrumb-link) {
  color: inherit;
  text-decoration: none;
}
.c-topbar :deep(.topbar-breadcrumb-link:hover) { text-decoration: underline; }

/* ── Mobile ──────────────────────────────────────────────────────── */
@media (max-width: 640px) {
  .c-topbar {
    padding: 0 var(--space-5);
  }

  /* Collapse sm buttons to square icon size */
  .c-topbar :deep(.c-btn--sm) {
    height: 36px;
    width: 36px;
  }
}
</style>
