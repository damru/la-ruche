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
      class="flex justify-between gap-3 py-3 px-4 rounded-xl bg-secondary-container toned"
      :class="['c-alert', toneClassMap[variant]]"
      role="alert"
    >
      <div class="flex flex-1 gap-2 min-w-0 ">
        <component
          :is="iconMap[variant]"
          class="icon-md"
          aria-hidden="true"
        />
        <div class="flex flex-col">
          <p v-if="title" class="text-md font-semibold">
            {{ title }}
          </p>
          <div class="text-sm">
            <slot></slot>
          </div>
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
  border-radius: var(--radius-lg);
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
