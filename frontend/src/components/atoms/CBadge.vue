<script setup lang="ts">
// Composant atomique — CBadge
// "Official stamp" chips for roles and statuses

type Variant =
  | 'manager' | 'board_member' | 'co_owner' | 'tenant'
  | 'open' | 'pending' | 'closed'
  | 'primary' | 'default'

withDefaults(defineProps<{
  variant?: Variant
  dot?: boolean
}>(), {
  variant: 'default',
  dot: false,
})
</script>

<template>
  <span :class="['c-badge', `c-badge--${variant}`, { 'tone--default': variant === 'primary', 'tone--warn': variant === 'pending' }]">
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
  padding: 4px 10px;
  border-radius: var(--radius-full);
  font-family: var(--font-body);
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.c-badge__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.65;
  flex-shrink: 0;
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
.c-badge--open    { background: var(--color-success-container); color: var(--color-success); }
.c-badge--pending { background: var(--tone-bg); color: var(--tone-fg); }
.c-badge--closed  { background: var(--color-surface-container-high); color: var(--color-on-surface-variant); }

/* ── Generic ───────────────────────────────────────────────── */
.c-badge--primary { background: var(--tone-bg); color: var(--tone-fg); }
.c-badge--default { background: var(--color-surface-container-high); color: var(--color-on-surface-variant); }
</style>
