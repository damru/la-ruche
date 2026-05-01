<script setup lang="ts">
// Atom — CTooltip
// Renders via Teleport to <body> with position:fixed — never overflows the screen.
// Arrow shifts to always point at the trigger center even when the popup is clamped.
import { ref, onUnmounted } from 'vue'

const props = defineProps<{
  content: string
  linkHref?: string
  linkLabel?: string
}>()

const visible = ref(false)
const triggerRef = ref<HTMLElement | null>(null)
const popupRef = ref<HTMLElement | null>(null)
let hideTimeout: ReturnType<typeof window.setTimeout> | null = null

// ── Computed position state ────────────────────────────────────────
const popupStyle = ref<Record<string, string>>({})
const arrowStyle = ref<Record<string, string>>({})
const arrowDir = ref<'up' | 'down'>('down')

const POPUP_W = 280
const ARROW_H = 10
const MARGIN = 10
const HIDE_DELAY_MS = 240

function clearHideTimeout() {
  if (hideTimeout !== null) {
    window.clearTimeout(hideTimeout)
    hideTimeout = null
  }
}

// ── Show / Hide ───────────────────────────────────────────────────
// The popup node is always in the DOM (v-show) so offsetHeight is
// immediately available — no rAF or nextTick needed.
function show() {
  clearHideTimeout()
  recalc()
  visible.value = true
  window.addEventListener('resize', recalc)
  window.addEventListener('scroll', recalc, { passive: true })
}

function scheduleHide() {
  clearHideTimeout()
  hideTimeout = window.setTimeout(() => {
    hide()
  }, HIDE_DELAY_MS)
}

function hide() {
  clearHideTimeout()
  visible.value = false
  cleanup()
}

function toggle() {
  if (visible.value) {
    hide()
    return
  }

  show()
}

function cleanup() {
  clearHideTimeout()
  window.removeEventListener('resize', recalc)
  window.removeEventListener('scroll', recalc)
}

onUnmounted(cleanup)

// ── Position calculation ───────────────────────────────────────────
function recalc() {
  if (!triggerRef.value || !popupRef.value) return

  const tr = triggerRef.value.getBoundingClientRect()
  const vw = window.innerWidth
  const vh = window.innerHeight
  const popH = popupRef.value.offsetHeight || 80

  // Horizontal: center on trigger, clamp to viewport
  const idealLeft = tr.left + tr.width / 2 - POPUP_W / 2
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
    left: `${clampedLeft}px`,
    top: `${top}px`,
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
      :aria-label="props.content"
      @mouseenter="show"
      @mouseleave="scheduleHide"
      @focus="show"
      @blur="scheduleHide"
      @click="toggle"
      @keydown.enter.prevent="toggle"
      @keydown.space.prevent="toggle"
      @keydown.escape.prevent="hide"
    >
      <slot>
        <svg
          class="c-tooltip-trigger__icon"
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <circle
            cx="12"
            cy="12"
            r="10"
            fill="currentColor"
          />
          <path
            d="M12 10.25v5.25"
            stroke="var(--color-surface)"
            stroke-width="2"
            stroke-linecap="round"
          />
          <circle
            cx="12"
            cy="7.25"
            r="1.25"
            fill="var(--color-surface)"
          />
        </svg>
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
        @mouseenter="show"
        @mouseleave="scheduleHide"
        @focusin="show"
        @focusout="scheduleHide"
      >
        <div class="c-tooltip-popup__content">
          {{ props.content }}
        </div>
        <template v-if="props.linkHref && props.linkLabel">
          <span
            class="c-tooltip-popup__separator"
            aria-hidden="true"
          ></span>
          <a
            class="c-tooltip-popup__link"
            :href="props.linkHref"
            target="_blank"
            rel="noreferrer noopener"
          >
            {{ props.linkLabel }}
          </a>
        </template>
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
.c-tooltip-trigger__icon {
  display: inline-flex;
  width: 14px;
  height: 14px;
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

.c-tooltip-popup__content {
  margin: 0;
}

.c-tooltip-popup__separator {
  display: block;
  margin: 10px 0 8px;
  border-top: 1px solid var(--color-outline-variant);
}

.c-tooltip-popup__link {
  color: var(--color-primary);
  font-weight: var(--fw-semibold);
  text-decoration: none;
}

.c-tooltip-popup__link:hover,
.c-tooltip-popup__link:focus-visible {
  text-decoration: underline;
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
