<script setup lang="ts">
import type { Component } from 'vue'

defineOptions({ inheritAttrs: false })

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'reversed'
type Size = 'sm' | 'md' | 'lg'
type Breakpoint = 'sm' | 'md' | 'lg' | 'xl' | '2xl'

withDefaults(defineProps<{
  variant?: Variant
  size?: Size
  disabled?: boolean
  loading?: boolean
  closing?: boolean
  type?: 'button' | 'submit' | 'reset'
  title?: string
  icon?: Component | string
  shrink?: Breakpoint
}>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  loading: false,
  closing: false,
  type: 'button',
  title: undefined,
  icon: undefined,
  shrink: undefined,
})

const variantClass: Record<Variant, string> = {
  primary: 'bg-(image:--tone-bg) enabled:hover:-translate-y-px enabled:hover:shadow-(--shadow-primary-lift) enabled:active:translate-y-0 enabled:active:shadow-(--shadow-primary-lift-hover)',
  secondary: 'enabled:hover:brightness-95',
  ghost: 'bg-transparent! text-primary! enabled:hover:bg-secondary-container',
  danger: 'enabled:hover:brightness-95',
  reversed: 'bg-(--tone-fg)! text-(--tone-bg)! enabled:hover:-translate-y-px',
}

const sizeClass: Record<Size, string> = {
  sm: 'min-h-9 min-w-9 px-3 text-sm rounded-lg [--c-btn-box:2.25rem]',
  md: 'min-h-11 min-w-11 px-5 text-base rounded-xl [--c-btn-box:2.75rem]',
  lg: 'min-h-[3.25rem] min-w-[3.25rem] px-6 text-lg rounded-2xl [--c-btn-box:3.25rem]',
}

const shrinkClass: Record<Breakpoint, string> = {
  sm: 'max-sm:w-(--c-btn-box) max-sm:min-w-(--c-btn-box) max-sm:px-0 max-sm:gap-0 max-sm:[&_span]:hidden',
  md: 'max-md:w-(--c-btn-box) max-md:min-w-(--c-btn-box) max-md:px-0 max-md:gap-0 max-md:[&_.c-btn__title]:hidden',
  lg: 'max-lg:w-(--c-btn-box) max-lg:min-w-(--c-btn-box) max-lg:px-0 max-lg:gap-0 max-lg:[&_.c-btn__title]:hidden',
  xl: 'max-xl:w-(--c-btn-box) max-xl:min-w-(--c-btn-box) max-xl:px-0 max-xl:gap-0 max-xl:[&_.c-btn__title]:hidden',
  '2xl': 'max-2xl:w-(--c-btn-box) max-2xl:min-w-(--c-btn-box) max-2xl:px-0 max-2xl:gap-0 max-2xl:[&_.c-btn__title]:hidden',
}

</script>

<template>
  <button
    :disabled="disabled"
    :type="type"
    :class="[
      variant ? variantClass[variant] : null, sizeClass[size], 
      { 
        'pointer-events-none': loading, 
        'overflow-clip': closing, 
        'tone--accent': variant === 'primary', 
        'tone--err': variant === 'danger', 
        'tone--disabled': disabled, 
        'c-btn--shrink': !!shrink 
      }, 
      shrink ? shrinkClass[shrink] : null
    ]"
    class= "inline-flex items-center justify-center gap-2 font-sans font-medium text-left w-fit overflow-hidden bg-(--tone-bg,var(--color-primary-container)) text-(--tone-fg,var(--color-on-primary)) [transition:transform_var(--transition-fast),box-shadow_var(--transition-fast),background_var(--transition-fast),filter_var(--transition-fast)] motion-reduce:transition-none focus-visible:outline-2 focus-visible:outline focus-visible:outline-[var(--tone-fg,var(--color-primary))] focus-visible:outline-offset-2 disabled:cursor-not-allowed!"
    v-bind="$attrs"
  >
    <template v-if="$slots.default">
      <slot></slot>
    </template>
    <template v-else>
      <span
        v-if="loading"
        class="c-btn__spinner"
        aria-hidden="true"
      ></span>
      <component
        :is="icon"
        v-else-if="icon"
        class="c-btn__icon"
        aria-hidden="true"
      />
      <span
        v-if="title"
        class="c-btn__title"
      >
        {{ title }}
      </span>
    </template>
  </button>
</template>

<style scoped>
/* ── Spinner ──────────────────────────────────────────────────── */
.c-btn__spinner {
  width: var(--icon-sm);
  height: var(--icon-sm);
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  flex-shrink: 0;
}

.c-btn__icon {
  display: block;
  width: var(--icon-sm);
  height: var(--icon-sm);
  flex-shrink: 0;
}

/* ── Closing progress fill (DDR-006) ──────────────────────────── */
/* Fills the button background from left to right over SUCCESS_AUTO_CLEAR_MS (3 s)
   to communicate the auto-close countdown after a successful action. */
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
  display: block !important;
  width: var(--icon-sm);
  height: var(--icon-sm);
  flex-shrink: 0;
}

@media (max-width: 640px) {
  :slotted([shrink]) {
    display: none;
  }
}

@media (max-width: 639.98px) {
  .c-btn--shrink-sm {
    width: var(--c-btn-box);
    min-width: var(--c-btn-box);
    padding-inline: 0;
    gap: 0;
  }

  .c-btn--shrink-sm .c-btn__title {
    display: none;
  }
}

@media (max-width: 767.98px) {
  .c-btn--shrink-md {
    width: var(--c-btn-box);
    min-width: var(--c-btn-box);
    padding-inline: 0;
    gap: 0;
  }

  .c-btn--shrink-md .c-btn__title {
    display: none;
  }
}

@media (max-width: 1023.98px) {
  .c-btn--shrink-lg {
    width: var(--c-btn-box);
    min-width: var(--c-btn-box);
    padding-inline: 0;
    gap: 0;
  }

  .c-btn--shrink-lg .c-btn__title {
    display: none;
  }
}

@media (max-width: 1279.98px) {
  .c-btn--shrink-xl {
    width: var(--c-btn-box);
    min-width: var(--c-btn-box);
    padding-inline: 0;
    gap: 0;
  }

  .c-btn--shrink-xl .c-btn__title {
    display: none;
  }
}

@media (max-width: 1535.98px) {
  .c-btn--shrink-2xl {
    width: var(--c-btn-box);
    min-width: var(--c-btn-box);
    padding-inline: 0;
    gap: 0;
  }

  .c-btn--shrink-2xl .c-btn__title {
    display: none;
  }
}

@media (prefers-reduced-motion: reduce) {
  .c-btn__spinner,
  .c-btn--closing::before {
    animation: none;
  }
}
</style>
