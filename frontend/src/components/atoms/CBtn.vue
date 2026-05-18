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
  /** When true, disables the button and shows a left-to-right fill progress animation
   *  (DDR-006: used during the auto-close timer after a successful modal action). */
  closing?: boolean
  type?: 'button' | 'submit' | 'reset'
}>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  loading: false,
  closing: false,
  type: 'button',
})
</script>

<template>
  <button
    :type="type"
    :disabled="disabled || loading"
    :class="['c-btn', `c-btn--${variant}`, `c-btn--${size}`, { 'c-btn--loading': loading, 'c-btn--closing': closing, 'tone--accent': variant === 'primary', 'tone--err': variant === 'danger' }]"
    v-bind="$attrs"
  >
    <span
      v-if="loading"
      class="c-btn__spinner"
      aria-hidden="true"
    ></span>
    <slot></slot>
  </button>
</template>

<style scoped>
.c-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  font-family: var(--font-body);
  font-weight: var(--fw-medium);
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
  text-align: left;
  text-wrap: auto;
  width: fit-content;
}

.c-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
  pointer-events: none;
}

/* ── Tailles ──────────────────────────────────────────────────── */
.c-btn--sm  { min-height: 36px; min-width: 36px; padding: 0 var(--space-3); font-size: var(--text-sm); border-radius: var(--radius-sm); }
.c-btn--md  { min-height: 44px; min-width: 44px; padding: 0 var(--space-5); font-size: var(--text-md); }
.c-btn--lg  { min-height: 52px; min-width: 52px; padding: 0 var(--space-6); font-size: var(--text-lg); }
.c-btn--icon { width: 44px; height: 44px; padding: 0; border-radius: var(--radius-sm); }

/* ── Variantes ────────────────────────────────────────────────── */
.c-btn--primary {
  background: var(--tone-bg);
  color: var(--tone-fg);
  box-shadow: var(--shadow-primary-lift);
}
.c-btn--primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: var(--shadow-primary-lift-hover);
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
  width: var(--icon-sm);
  height: var(--icon-sm);
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  flex-shrink: 0;
}

/* ── Closing progress fill (DDR-006) ──────────────────────────── */
/* Fills the button background from left to right over SUCCESS_AUTO_CLEAR_MS (3 s)
   to communicate the auto-close countdown after a successful action. */
.c-btn--closing {
  overflow: clip;
}
.c-btn--closing::before {
  content: '';
  position: absolute;
  inset: 0;
  background: rgba(255, 255, 255, 0.25);
  transform-origin: left center;
  transform: scaleX(0);
  animation: c-btn-fill 3s linear forwards;
}

@keyframes c-btn-fill {
  from { transform: scaleX(0); }
  to   { transform: scaleX(1); }
}

/* ── Slot icons ───────────────────────────────────────────────── */
:slotted(svg) {
  width: var(--icon-sm);
  height: var(--icon-sm);
  flex-shrink: 0;
}
</style>
