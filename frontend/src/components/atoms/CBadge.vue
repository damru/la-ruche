<script setup lang="ts">
// Composant atomique — CBadge
// "Official stamp" chips for roles and statuses

import { computed } from 'vue'

type Variant =
  | 'manager' | 'board_member' | 'co_owner' | 'tenant'
  | 'open' | 'pending' | 'closed' | 'disabled'
  | 'valid' | 'warn' | 'err'
  | 'primary' | 'default' | 'accent'

const props = withDefaults(defineProps<{
  variant?: Variant
  dot?: boolean
  ring?: boolean
  pulse?: boolean
  truncate?: boolean
}>(), {
  variant: 'default',
  dot: false,
  ring: false,
  pulse: undefined,
  truncate: false,
})

const effectivePulse = computed(() =>
  props.pulse ?? (props.variant === 'valid' || props.variant === 'open')
)
</script>

<template>
  <span
    :class="['c-badge', `c-badge--${variant}`, {
      'c-badge--pulse': effectivePulse,
      'c-badge--ring': ring,
      'c-badge--truncate': truncate,
      'tone--manager': variant === 'manager',
      'tone--board-member': variant === 'board_member',
      'tone--co-owner': variant === 'co_owner',
      'tone--tenant': variant === 'tenant',
      'tone--default': variant === 'primary' || variant === 'default',
      'tone--accent': variant === 'accent',
      'tone--disabled': variant === 'disabled' || variant === 'closed',
      'tone--valid': variant === 'valid' || variant === 'open',
      'tone--warn': variant === 'pending' || variant === 'warn',
      'tone--err': variant === 'err',
    }]"
  >
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
  font-weight: var(--fw-medium);
  white-space: nowrap;
  width: fit-content;
  height: fit-content;
  background: var(--tone-bg);
  color: var(--tone-fg);
}

/* Truncate long labels (e.g. calendar event titles) within the parent width.
   `display: block` is required: text-overflow only ellipsizes block containers,
   and inline-flex clips without the "…". */
.c-badge--truncate {
  display: block;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
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

/* Empty ring dot for closed / inactive / disabled statuses or explicit ring */
.c-badge--ring .c-badge__dot,
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

</style>
