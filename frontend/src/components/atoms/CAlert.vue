<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import CIconInfo from '@/components/atoms/CIconInfo.svg'
import CIconWarn from '@/components/atoms/CIconWarn.svg'
import CIconCheckCircle from '@/components/atoms/CIconCheckCircle.svg'
import CIconXCircle from '@/components/atoms/CIconXCircle.svg'
import CIconX from '@/components/atoms/CIconX.svg'

import { computed, ref, useSlots } from 'vue'

const { t } = useI18n()
const slots = useSlots()

type Variant = 'info' | 'warning' | 'error' | 'success'

withDefaults(defineProps<{
  variant?: Variant
  title?: string
  dismissible?: boolean
}>(), {
  variant: 'info',
  title: undefined,
  dismissible: false,
})

const emit = defineEmits<{ dismiss: [] }>()
const visible = ref(true)
const hasSideSlot = computed(() => Boolean(slots.side))

const iconMap: Record<Variant, typeof CIconInfo> = {
  info:    CIconInfo,
  warning: CIconWarn,
  error:   CIconXCircle,
  success: CIconCheckCircle,
}
const toneClassMap: Record<Variant, string> = {
  info: 'tone--info',
  warning: 'tone--warn',
  error: 'tone--err',
  success: 'tone--valid',
}

function dismiss() {
  visible.value = false
  emit('dismiss')
}
</script>

<template>
  <Transition name="c-alert-fade">
    <div
      v-if="visible"
      :class="['c-alert', toneClassMap[variant]]"
      role="alert"
    >
      <div class="c-alert__main">
        <div
          v-if="title"
          class="c-alert__header"
        >
          <component
            :is="iconMap[variant]"
            class="c-alert__icon"
            aria-hidden="true"
          />
          <p class="c-alert__title">
            {{ title }}
          </p>
        </div>
        <div
          v-else
          class="c-alert__header"
        >
          <component
            :is="iconMap[variant]"
            class="c-alert__icon"
            aria-hidden="true"
          />
        </div>
        <div class="c-alert__body">
          <slot></slot>
        </div>
      </div>
      <div
        v-if="hasSideSlot || dismissible"
        :class="[
          'c-alert__side',
          { 'c-alert__side--with-close': dismissible },
        ]"
      >
        <button
          v-if="dismissible"
          class="c-alert__close"
          :aria-label="t('common.close')"
          @click="dismiss"
        >
          <CIconX
            class="icon-sm"
            aria-hidden="true"
          />
        </button>
        <div
          v-if="hasSideSlot"
          class="c-alert__side-content"
        >
          <slot name="side"></slot>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.c-alert {
  display: flex;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
  background: var(--tone-bg, var(--color-secondary-container));
  color: var(--tone-fg, var(--color-on-secondary-container));
}

.c-alert__main {
  flex: 1;
  min-width: 0;
  align-self: flex-start;
}

.c-alert__header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.c-alert__icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

.c-alert__body {
  min-width: 0;
  margin-top: 2px;
}

.c-alert__title {
  font-size: var(--text-sm);
  font-weight: var(--fw-semibold);
  margin: 0;
}

.c-alert__body :deep(p) {
  margin: 0;
}

.c-alert__body {
  font-size: var(--text-sm);
  opacity: 0.9;
  line-height: 1.4;
}

.c-alert__side {
  display: flex;
  flex-shrink: 0;
  align-self: stretch;
}

.c-alert__side--with-close {
  display: grid;
  grid-template-rows: auto 1fr;
  justify-items: end;
  align-items: stretch;
  gap: var(--space-2);
}

.c-alert__side-content {
  display: flex;
  min-height: 0;
}

.c-alert__side--with-close .c-alert__side-content {
  justify-self: stretch;
}

.c-alert__close {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: unset;
  min-height: unset;
  border-radius: var(--radius-sm);
  opacity: 0.6;
  transition: opacity var(--transition-fast);
  justify-self: end;
  align-self: start;
}
.c-alert__close:hover { opacity: 1; }

/* ── Transition ────────────────────────────────────────────────── */
.c-alert-fade-enter-active { transition: all var(--transition-base); }
.c-alert-fade-leave-active { transition: all var(--transition-fast); }
.c-alert-fade-enter-from, .c-alert-fade-leave-to { opacity: 0; transform: translateY(-4px); }
</style>
