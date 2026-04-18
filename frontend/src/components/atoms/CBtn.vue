<script setup lang="ts">
// Composant atomique — CBtn
// Respecte le design system : gradient primary, ghost, secondary, danger
// Touch target min 44px garanti par CSS base

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger'
type Size = 'sm' | 'md' | 'lg' | 'icon'

withDefaults(defineProps<{
  variant?: Variant
  size?: Size
  disabled?: boolean
  loading?: boolean
  type?: 'button' | 'submit' | 'reset'
}>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  loading: false,
  type: 'button',
})
</script>

<template>
  <button
    :type="type"
    :disabled="disabled || loading"
    :class="['c-btn', `c-btn--${variant}`, `c-btn--${size}`, { 'c-btn--loading': loading, 'tone--accent': variant === 'primary', 'tone--err': variant === 'danger' }]"
    v-bind="$attrs"
  >
    <span v-if="loading" class="c-btn__spinner" aria-hidden="true" />
    <slot />
  </button>
</template>

<style scoped>
.c-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  font-family: var(--font-body);
  font-weight: 500;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition:
    transform var(--transition-fast),
    box-shadow var(--transition-fast),
    background var(--transition-fast),
    filter var(--transition-fast);
  white-space: nowrap;
  position: relative;
  text-decoration: none;
}

.c-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
  pointer-events: none;
}

/* ── Tailles ──────────────────────────────────────────────────── */
.c-btn--sm  { height: 36px; padding: 0 var(--space-3); font-size: 13px; border-radius: var(--radius-sm); }
.c-btn--md  { height: 44px; padding: 0 var(--space-5); font-size: 14px; }
.c-btn--lg  { height: 52px; padding: 0 var(--space-6); font-size: 16px; }
.c-btn--icon { width: 44px; height: 44px; padding: 0; border-radius: var(--radius-sm); }

/* ── Variantes ────────────────────────────────────────────────── */
.c-btn--primary {
  background: var(--tone-bg);
  color: var(--tone-fg);
  box-shadow: 0 4px 16px rgba(100, 63, 213, 0.25);
}
.c-btn--primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 8px 24px rgba(100, 63, 213, 0.35);
}
.c-btn--primary:active:not(:disabled) {
  transform: translateY(0);
}

.c-btn--secondary {
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
}
.c-btn--secondary:hover:not(:disabled) {
  filter: brightness(0.96);
}

.c-btn--ghost {
  background: transparent;
  color: var(--color-primary);
}
.c-btn--ghost:hover:not(:disabled) {
  background: var(--color-secondary-container);
}

.c-btn--danger {
  background: var(--tone-bg);
  color: var(--tone-fg);
}
.c-btn--danger:hover:not(:disabled) {
  filter: brightness(0.96);
}

/* ── Spinner ──────────────────────────────────────────────────── */
.c-btn--loading { pointer-events: none; }
.c-btn__spinner {
  width: 16px;
  height: 16px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  flex-shrink: 0;
}

/* ── Slot icons ───────────────────────────────────────────────── */
:slotted(svg) {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}
</style>
