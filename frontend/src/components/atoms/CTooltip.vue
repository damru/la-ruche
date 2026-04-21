<script setup lang="ts">
// Atom — CTooltip
// Renders via Teleport to <body> with position:fixed — never overflows the screen.
// Arrow shifts to always point at the trigger center even when the popup is clamped.
import { ref, onUnmounted } from 'vue'

defineProps<{ content: string }>()

const visible = ref(false)
const triggerRef = ref<HTMLElement | null>(null)
const popupRef  = ref<HTMLElement | null>(null)

// ── Computed position state ────────────────────────────────────────
const popupStyle  = ref<Record<string, string>>({})
const arrowStyle  = ref<Record<string, string>>({})
const arrowDir    = ref<'up' | 'down'>('down')

const POPUP_W  = 260
const ARROW_H  = 10  // tip-to-base height of the CSS triangle
const MARGIN   = 10  // minimum distance from screen edges

// ── Show / Hide ───────────────────────────────────────────────────
// The popup node is always in the DOM (v-show) so offsetHeight is
// immediately available — no rAF or nextTick needed.
function show() {
  recalc()
  visible.value = true
  window.addEventListener('resize', recalc)
  window.addEventListener('scroll', recalc, { passive: true })
}
function hide() {
  visible.value = false
  cleanup()
}

function cleanup() {
  window.removeEventListener('resize', recalc)
  window.removeEventListener('scroll', recalc)
}

onUnmounted(cleanup)

// ── Position calculation ───────────────────────────────────────────
function recalc() {
  if (!triggerRef.value || !popupRef.value) return

  const tr  = triggerRef.value.getBoundingClientRect()
  const vw  = window.innerWidth
  const vh  = window.innerHeight
  const popH = popupRef.value.offsetHeight || 80

  // Horizontal: center on trigger, clamp to viewport
  const idealLeft   = tr.left + tr.width / 2 - POPUP_W / 2
  const clampedLeft = Math.max(MARGIN, Math.min(idealLeft, vw - POPUP_W - MARGIN))

  // Vertical: prefer above, flip below if not enough space
  const spaceAbove = tr.top - MARGIN
  const spaceBelow = vh - tr.bottom - MARGIN
  let top: number

  if (spaceAbove >= popH + ARROW_H) {
    top = tr.top - popH - ARROW_H
    arrowDir.value = 'down'
  } else if (spaceBelow >= popH + ARROW_H) {
    top = tr.bottom + ARROW_H
    arrowDir.value = 'up'
  } else {
    // Neither side fits — pick the larger one
    top = spaceAbove >= spaceBelow
      ? Math.max(MARGIN, tr.top - popH - ARROW_H)
      : tr.bottom + ARROW_H
    arrowDir.value = spaceAbove >= spaceBelow ? 'down' : 'up'
  }

  // Clamp vertically
  top = Math.max(MARGIN, Math.min(top, vh - popH - MARGIN))

  popupStyle.value = {
    left:  `${clampedLeft}px`,
    top:   `${top}px`,
    width: `${POPUP_W}px`,
  }

  // Arrow: always points to trigger center, accounting for popup clamping
  const triggerCenterX = tr.left + tr.width / 2
  const arrowLeft = triggerCenterX - clampedLeft
  const arrowLeftClamped = Math.max(12, Math.min(arrowLeft, POPUP_W - 12))
  arrowStyle.value = { left: `${arrowLeftClamped}px` }
}
</script>

<template>
  <span class="c-tooltip-wrap">
    <button
      ref="triggerRef"
      type="button"
      class="c-tooltip-trigger"
      :class="{ 'c-tooltip-trigger--active': visible }"
      :aria-expanded="visible"
      :aria-label="content"
      @mouseenter="show"
      @mouseleave="hide"
      @focus="show"
      @blur="hide"
    >
      <slot>
        <!-- Default: info circle icon -->
        <CIconInfo aria-hidden="true" />
      </slot>
    </button>

    <Teleport to="body">
      <div
        ref="popupRef"
        class="c-tooltip-popup"
        :class="[`c-tooltip-popup--arrow-${arrowDir}`, { 'c-tooltip-popup--hidden': !visible }]"
        :style="popupStyle"
        role="tooltip"
        :aria-hidden="!visible"
      >
        {{ content }}
        <span
          class="c-tooltip-arrow"
          :style="arrowStyle"
        ></span>
      </div>
    </Teleport>
  </span>
</template>

<style scoped>
/* ── Trigger ──────────────────────────────────────────────────── */
.c-tooltip-wrap {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  align-self: center;
}
.c-tooltip-trigger {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  min-width: unset;  /* opt-out of base.css 44px touch-target — icon embedded in a larger interactive zone */
  min-height: unset;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--color-on-surface-variant);
  opacity: 0.6;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: opacity var(--transition-fast), color var(--transition-fast);
  flex-shrink: 0;
}
.c-tooltip-trigger:hover,
.c-tooltip-trigger--active {
  opacity: 1;
  color: var(--color-primary);
}
:slotted(svg) { width: 16px; height: 16px; }
</style>

<!-- Global styles for teleported popup (not scoped) -->
<style>
.c-tooltip-popup {
  position: fixed;
  z-index: var(--z-modal);
  padding: 10px 14px;
  background: var(--color-secondary-container);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-md);
  font-family: var(--font-body);
  font-size: 12px;
  font-weight: var(--fw-medium);
  line-height: 1.55;
  color: var(--color-on-surface);
  box-shadow: var(--shadow-md);
}

.c-tooltip-popup--hidden {
  visibility: hidden;
  pointer-events: none;
}

/* Thick anchor border on the side facing the trigger */
.c-tooltip-popup--arrow-down {
  border-bottom: 3px solid var(--color-primary);
}
.c-tooltip-popup--arrow-up {
  border-top: 3px solid var(--color-primary);
}

/* Arrow — solid primary triangle extending from the thick border */
.c-tooltip-arrow {
  position: absolute;
  left: 50%; /* overridden by JS */
  width: 0;
  height: 0;
  transform: translateX(-50%);
  border-style: solid;
}

/* Arrow pointing DOWN (popup above trigger) — exits from thick bottom border */
.c-tooltip-popup--arrow-down .c-tooltip-arrow {
  top: 100%;
  margin-top: 2px; /* overlap the thick border slightly */
  border-width: 7px 6px 0 6px;
  border-color: var(--color-primary) transparent transparent transparent;
}

/* Arrow pointing UP (popup below trigger) — exits from thick top border */
.c-tooltip-popup--arrow-up .c-tooltip-arrow {
  bottom: 100%;
  margin-bottom: 2px;
  border-width: 0 6px 7px 6px;
  border-color: transparent transparent var(--color-primary) transparent;
}
</style>
