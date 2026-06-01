<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Component } from 'vue'

defineOptions({ inheritAttrs: false })

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'negative' | 'icon'
type Size = 'xs' | 'sm' | 'md' | 'lg'
type Breakpoint = 'sm' | 'md' | 'lg' | 'xl' | '2xl'

const props = withDefaults(defineProps<{
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

const { t } = useI18n()

const MIN_LOADING_MS = 300
const visualLoading = ref(props.loading)
let loadingStart = 0

watch(() => props.loading, (val) => {
  if (val) {
    loadingStart = Date.now()
    visualLoading.value = true
  } else {
    const remaining = MIN_LOADING_MS - (Date.now() - loadingStart)
    if (remaining > 0) {
      setTimeout(() => { visualLoading.value = false }, remaining)
    } else {
      visualLoading.value = false
    }
  }
})

const variantClass: Record<Variant, string> = {
  primary: 'c-btn--primary',
  secondary: 'c-btn--secondary',
  ghost: 'c-btn--ghost',
  danger: 'c-btn--danger',
  negative: 'c-btn--negative',
  icon: 'c-btn--icon-variant',
}

const sizeClass: Record<Size, string> = {
  xs: 'c-btn--xs',
  sm: 'c-btn--sm',
  md: 'c-btn--md',
  lg: 'c-btn--lg',
}

const iconSizeClass: Record<Size, string> = {
  xs: 'icon-xs',
  sm: 'icon-sm',
  md: 'icon-md',
  lg: 'icon-lg',
}

const shrinkClass: Record<Breakpoint, string> = {
  sm: 'c-btn--shrink-sm',
  md: 'c-btn--shrink-md',
  lg: 'c-btn--shrink-lg',
  xl: 'c-btn--shrink-xl',
  '2xl': 'c-btn--shrink-2xl',
}

</script>

<template>
  <button
    :disabled="disabled"
    :type="type"
    :class="[
      variantClass[variant], sizeClass[size],
      { 
        'pointer-events-none': loading || visualLoading, 
        'c-btn--closing': closing, 
        'tone--accent': variant === 'primary', 
        'tone--info': variant === 'secondary', 
        'tone--err': variant === 'danger', 
        'tone--disabled': disabled && !visualLoading, 
        'c-btn--shrink': !!shrink 
      }, 
      shrink ? shrinkClass[shrink] : null
    ]"
    class="c-btn"
    v-bind="$attrs"
  >
    <template v-if="$slots.default">
      <slot></slot>
    </template>
    <template v-else>
      <span
        v-if="visualLoading || closing"
        :class="{ 'c-btn__spinner--overlay': visualLoading }"
        class="c-btn__spinner"
        aria-hidden="true"
      ></span>
      <component
        :is="icon"
        v-if="icon && !closing"
        :class="[iconSizeClass[size], { 'c-btn__hidden': visualLoading }]"
        aria-hidden="true"
      />
      <span 
        v-if="title && variant !== 'icon'"
        :class="{ 'c-btn__hidden': visualLoading }"
        class="c-btn__title"
      >
        {{ closing ? t('common.closing') : title }}
      </span>
    </template>
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
  text-align: left;
  width: fit-content;
  min-width: 0;
  position: relative;
  overflow: hidden;
  border: none;
  background: var(--tone-bg, var(--color-primary-container));
  color: var(--tone-fg, var(--color-on-primary));
  cursor: pointer;
  transition:
    transform var(--transition-fast),
    box-shadow var(--transition-fast),
    background var(--transition-fast),
    filter var(--transition-fast);
}

.c-btn:disabled {
  cursor: not-allowed;
}

.c-btn:focus-visible {
  outline: 2px solid var(--tone-fg, var(--color-primary));
  outline-offset: 2px;
}

.c-btn--primary {
  background: var(--tone-bg);
}
  
.c-btn--primary:not(:disabled) {
  box-shadow: var(--shadow-primary-lift);
}

.c-btn--primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: var(--shadow-primary-lift-hover);
}

.c-btn--primary:active:not(:disabled) {
  transform: translateY(0);
}

.c-btn--secondary:hover:not(:disabled),
.c-btn--danger:hover:not(:disabled) {
  filter: brightness(0.95);
}

.c-btn--ghost {
  background: transparent;
  color: var(--tone-fg, var(--color-primary));
}

.c-btn--ghost:hover:not(:disabled) {
  background: var(--color-secondary-container);
}

.c-btn--negative {
  background: var(--tone-fg, var(--color-on-surface));
  color: var(--tone-bg, var(--color-surface));
}

.c-btn--negative:hover:not(:disabled) {
  transform: translateY(-1px);
}

.c-btn--negative:active:not(:disabled) {
  transform: translateY(0);
}

.c-btn--icon-variant {
  width: var(--c-btn-box);
  min-width: min-content;
  min-height: min-content;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--tone-bg, var(--color-on-surface-variant));
  padding: 0 !important;
  gap: 0;
}

.c-btn--icon-variant:hover:not(:disabled) {
  background: var(--tone-bg, var(--color-surface-container));
  color: var(--tone-fg, var(--color-primary));
}

.c-btn--xs {
  min-height: 1.75rem;
  min-width: 1.75rem;
  padding: 0 0.5rem;
  font-size: var(--text-xs);
  border-radius: var(--radius-md);
  --c-btn-box: 1.75rem;
}

.c-btn--sm {
  min-height: 2.25rem;
  min-width: 2.25rem;
  padding: 0 0.75rem;
  font-size: var(--text-sm);
  border-radius: var(--radius-lg);
  --c-btn-box: 2.25rem;
}

.c-btn--md {
  min-height: 2.75rem;
  min-width: 2.75rem;
  padding: 0 1.25rem;
  font-size: var(--text-md);
  border-radius: var(--radius-xl);
  --c-btn-box: 2.75rem;
}

.c-btn--lg {
  min-height: 3.25rem;
  min-width: 3.25rem;
  padding: 0 1.5rem;
  font-size: var(--text-lg);
  border-radius: var(--radius-2xl);
  --c-btn-box: 3.25rem;
}

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

/* Centered overlay: shown alone when loading, preserving button size */
.c-btn__spinner--overlay {
  position: absolute;
  inset: 0;
  margin: auto;
}

.c-btn__hidden {
  visibility: hidden;
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

@media (max-width: 640px) {
  .c-btn--icon-variant, .c-btn--shrink-sm {
    width: var(--c-btn-box);
    min-width: var(--c-btn-box);
    padding-inline: 0;
    gap: 0;
  }

  .c-btn--shrink-sm .c-btn__title {
    display: none;
  }
}

@media (max-width: 768px) {
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

@media (max-width: 1024px) {
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

@media (max-width: 1280px) {
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

@media (max-width: 1536px) {
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
  .c-btn {
    transition: none;
  }

  .c-btn__spinner,
  .c-btn--closing::before {
    animation: none;
  }
}
</style>
