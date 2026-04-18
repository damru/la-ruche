<script setup lang="ts">
// Organism — CTopbar
// Sticky page topbar with glassmorphism effect
// Slots: #left (greeting or breadcrumb), #right (action buttons)
// Ref: DESIGN.md — used by all authenticated views
import { useUiStore } from '@/stores/ui'

const ui = useUiStore()
</script>

<template>
  <div class="c-topbar">
    <div class="c-topbar__left">
      <slot name="left" />
    </div>
    <div class="c-topbar__right">
      <slot name="right" />
      <!-- Theme toggle — always present -->
      <button
        class="c-topbar__theme-btn"
        :aria-label="ui.isDark ? 'Passer en mode clair' : 'Passer en mode sombre'"
        @click="ui.toggleTheme()"
      >
        <!-- Sun icon (visible in dark mode) -->
        <svg v-if="ui.isDark" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <circle cx="12" cy="12" r="5"/>
          <line x1="12" y1="1" x2="12" y2="3"/>
          <line x1="12" y1="21" x2="12" y2="23"/>
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
          <line x1="1" y1="12" x2="3" y2="12"/>
          <line x1="21" y1="12" x2="23" y2="12"/>
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
        </svg>
        <!-- Moon icon (visible in light mode) -->
        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M21 12.79A9 9 0 1111.21 3a7 7 0 009.79 9.79z"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.c-topbar {
  position: sticky;
  top: 0;
  z-index: var(--z-nav);
  height: 60px;
  padding: 0 var(--space-10);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  background: rgba(248, 249, 250, 0.88);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border-bottom: 1px solid rgba(194, 199, 207, 0.25);
  transition: background var(--transition-base);
}

[data-theme="dark"] .c-topbar {
  background: rgba(17, 19, 24, 0.88);
}

.c-topbar__left,
.c-topbar__right {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

/* Breadcrumb helper — for views using breadcrumb in left slot */
.c-topbar :deep(.topbar-breadcrumb) {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--color-on-surface-variant);
}

.c-topbar :deep(.topbar-breadcrumb span) {
  color: var(--color-on-surface);
  font-weight: var(--fw-medium);
}

.c-topbar :deep(.topbar-breadcrumb svg) {
  width: 14px;
  height: 14px;
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
  font-size: 12px;
  color: var(--color-on-surface-variant);
}

/* Theme toggle button */
.c-topbar__theme-btn {
  width: 38px;
  height: 38px;
  min-width: unset;
  min-height: unset;
  border-radius: var(--radius-full);
  background: var(--color-surface-container);
  color: var(--color-on-surface-variant);
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.c-topbar__theme-btn:hover {
  background: var(--color-surface-container-high);
}

.c-topbar__theme-btn svg {
  width: 16px;
  height: 16px;
}
</style>
