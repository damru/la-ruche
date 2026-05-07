<script setup lang="ts">
// Composant atomique — CBadge
// "Official stamp" chips for roles and statuses

import { computed } from 'vue'

type Variant =
  | 'manager' | 'board_member' | 'co_owner' | 'tenant'
  | 'open' | 'pending' | 'closed' | 'disabled'
  | 'valid' | 'err'
  | 'primary' | 'default'

const props = withDefaults(defineProps<{
  variant?: Variant
  dot?: boolean
  // pulse defaults to true when variant is 'valid' or 'open', false otherwise
  pulse?: boolean
}>(), {
  variant: 'default',
  dot: false,
  pulse: undefined,
})

const effectivePulse = computed(() =>
  props.pulse ?? (props.variant === 'valid' || props.variant === 'open')
)
</script>

<template>
  <span :class="['c-badge', `c-badge--${variant}`, { 'c-badge--pulse': effectivePulse, 'tone--default': variant === 'primary', 'tone--warn': variant === 'pending', 'tone--valid': variant === 'valid' || variant === 'open', 'tone--err': variant === 'err', 'tone--disabled': variant === 'disabled' }]">
    <span
      v-if="dot"
      class="c-badge__dot"
      aria-hidden="true"
    ></span>
    <slot></slot>
  </span>
</template>

<style scoped>
.c-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-full);
  font-family: var(--font-body);
  font-size: var(--text-xs);
  font-weight: var(--fw-medium);
  white-space: nowrap;
  width: fit-content;
}

.c-badge__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.65;
  flex-shrink: 0;
}

/* Blinking dot — controlled by the pulse prop */
.c-badge--pulse .c-badge__dot {
  animation: badge-pulse 2.8s ease-in-out infinite;
}

@media (prefers-reduced-motion: reduce) {
  .c-badge--pulse .c-badge__dot {
    animation: none;
  }
}

/* Empty ring dot for closed / inactive / disabled statuses */
.c-badge--closed .c-badge__dot,
.c-badge--disabled .c-badge__dot {
  background: transparent;
  border: 1.5px solid currentColor;
  opacity: 0.5;
}

@keyframes badge-pulse {
  0%, 100% { opacity: 0.65; transform: scale(1); }
  50%       { opacity: 0.2;  transform: scale(0.85); }
}

/* ── Roles ──────────────────────────────────────────────────── */
.c-badge--manager      { background: #e8def8; color: #21005d; }
.c-badge--board_member { background: #d2e4fb; color: #001d36; }
.c-badge--co_owner     { background: #cef5e5; color: #00391d; }
.c-badge--tenant       { background: #fcecd1; color: #3a2000; }

[data-theme="dark"] .c-badge--manager      { background: #4a4458; color: #e8def8; }
[data-theme="dark"] .c-badge--board_member { background: #00345b; color: #d2e4fb; }
[data-theme="dark"] .c-badge--co_owner     { background: #00522b; color: #cef5e5; }
[data-theme="dark"] .c-badge--tenant       { background: #573200; color: #fcecd1; }

/* ── Statuts ──────────────────────────────────────────────────── */
.c-badge--open    { background: var(--tone-bg); color: var(--tone-fg); }
.c-badge--pending { background: var(--tone-bg); color: var(--tone-fg); }
.c-badge--closed  { background: var(--color-surface-container-high); color: var(--color-on-surface-variant); }
.c-badge--disabled { background: var(--tone-bg); color: var(--tone-fg); }

/* ── Priority / Urgency ────────────────────────────────────── */
.c-badge--valid { background: var(--tone-bg); color: var(--tone-fg); }
.c-badge--err   { background: var(--tone-bg); color: var(--tone-fg); }

/* ── Generic ───────────────────────────────────────────────── */
.c-badge--primary { background: var(--tone-bg); color: var(--tone-fg); }
.c-badge--default { background: var(--color-surface-container-high); color: var(--color-on-surface-variant); }
</style>
