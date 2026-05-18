<script setup lang="ts">
// Molecule component — CModal
// Glassmorphism (backdrop-filter blur) — DDR-002
// Accessibility: Radix Vue Dialog handles focus trap, aria, and escape

import { computed } from 'vue'
import { DialogRoot, DialogPortal, DialogOverlay, DialogContent, DialogTitle } from 'radix-vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = withDefaults(defineProps<{
  open: boolean
  title?: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
  closable?: boolean
  stack?: number
}>(), {
  title: undefined,
  size: 'md',
  closable: true,
  stack: 0,
})

const emit = defineEmits<{ 'update:open': [value: boolean] }>()

const stackStyle = computed(() => ({
  '--c-modal-stack': String(props.stack),
}))

function close() {
  if (props.closable) emit('update:open', false)
}

function handleOpenChange(open: boolean) {
  if (!open) close()
  else emit('update:open', true)
}
</script>

<template>
  <DialogRoot
    :open="open"
    @update:open="handleOpenChange"
  >
    <DialogPortal>
      <DialogOverlay
        class="c-modal__overlay"
        :style="stackStyle"
        @click="close"
      />
      <DialogContent
        :class="['c-modal__content', `c-modal__content--${size}`]"
        :style="stackStyle"
        @escape-key-down="close"
        @pointer-down-outside="e => { e.preventDefault(); close() }"
      >
        <!-- Header -->
        <div
          v-if="title || $slots.header"
          class="c-modal__header"
        >
          <DialogTitle
            as="h2"
            class="c-modal__title"
          >
            <slot name="header">
              {{ title }}
            </slot>
          </DialogTitle>
          <button
            class="c-modal__close"
            :class="{ 'c-modal__close--disabled': !closable }"
            :aria-label="t('common.close')"
            @click="close"
          >
            <CIconX
              width="18"
              height="18"
              aria-hidden="true"
            />
          </button>
        </div>

        <!-- Corps -->
        <div class="c-modal__body">
          <slot></slot>
        </div>

        <!-- Footer -->
        <div
          v-if="$slots.footer"
          class="c-modal__footer"
        >
          <slot name="footer"></slot>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.c-modal__overlay {
  position: fixed;
  inset: 0;
  background: rgba(25, 28, 29, 0.4);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  -moz-backdrop-filter: blur(4px);
  backdrop-filter: blur(4px);
  z-index: calc(var(--z-overlay) + var(--c-modal-stack, 0));
  animation: c-overlay-in 0.2s ease;
}

[data-theme="dark"] .c-modal__overlay {
  background: rgba(0, 0, 0, 0.6);
}

.c-modal__content {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: calc(var(--z-modal) + var(--c-modal-stack, 0));

  /* Glassmorphism */
  background: rgba(255, 255, 255, 0.85);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  -moz-backdrop-filter: blur(20px);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(194, 199, 207, 0.3);
  box-shadow: var(--shadow-glass);
  border-radius: var(--radius-lg);
  overflow: clip;

  max-height: 90vh;
  display: flex;
  flex-direction: column;

  animation: c-modal-in 0.25s cubic-bezier(0.34, 1.3, 0.64, 1);
}

[data-theme="dark"] .c-modal__content {
  background: rgba(28, 30, 38, 0.9);
  border-color: rgba(67, 71, 79, 0.5);
}

.c-modal__content--sm { width: min(340px, 90vw); }
.c-modal__content--md { width: min(480px, 90vw); }
.c-modal__content--lg { width: min(680px, 90vw); }
.c-modal__content--xl { width: min(1120px, 96vw); }

.c-modal__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-6) var(--space-6) 0;
  flex-shrink: 0;
}

.c-modal__title {
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  letter-spacing: -0.01em;
  color: var(--color-on-surface);
  line-height: 1.2;
}

.c-modal__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  min-width: unset;
  min-height: unset;
  border-radius: var(--radius-sm);
  color: var(--color-on-surface-variant);
  flex-shrink: 0;
  transition: background var(--transition-fast), color var(--transition-fast);
}
.c-modal__close:hover {
  background: var(--color-surface-container-high);
  color: var(--color-on-surface);
}
.c-modal__close--disabled {
  opacity: 0.35;
  cursor: not-allowed;
  pointer-events: none;
}

.c-modal__body {
  padding: var(--space-5) var(--space-6);
  overflow-y: auto;
  flex: 1;
}

.c-modal__footer {
  padding: 0 var(--space-6) var(--space-5);
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  flex-shrink: 0;
}

/* ── Animations ───────────────────────────────────────────────── */
@keyframes c-overlay-in {
  from { opacity: 0; }
  to   { opacity: 1; }
}

@keyframes c-modal-in {
  from { opacity: 0; transform: translate(-50%, calc(-50% + 12px)) scale(0.97); }
  to   { opacity: 1; transform: translate(-50%, -50%) scale(1); }
}
</style>
