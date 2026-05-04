<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import CIconInfo from '@/components/atoms/CIconInfo.svg'
import CIconWarn from '@/components/atoms/CIconWarn.svg'
import CIconAlertTriangle from '@/components/atoms/CIconAlertTriangle.svg'
import CIconCheckCircle from '@/components/atoms/CIconCheckCircle.svg'
import CIconX from '@/components/atoms/CIconX.svg'

import { ref } from 'vue'

const { t } = useI18n()

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

const iconMap: Record<Variant, typeof CIconInfo> = {
  info:    CIconInfo,
  warning: CIconWarn,
  error:   CIconAlertTriangle,
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
      <component
        :is="iconMap[variant]"
        class="c-alert__icon"
        aria-hidden="true"
      />
      <div class="c-alert__body">
        <p
          v-if="title"
          class="c-alert__title"
        >
          {{ title }}
        </p>
        <p class="c-alert__text">
          <slot></slot>
        </p>
      </div>
      <button
        v-if="dismissible"
        class="c-alert__close"
        :aria-label="t('common.close')"
        @click="dismiss"
      >
        <CIconX
          width="16"
          height="16"
          aria-hidden="true"
        />
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.c-alert {
  display: flex;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
  background: var(--tone-bg, var(--color-secondary-container));
  color: var(--tone-fg, var(--color-on-secondary-container));
}

.c-alert__icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  margin-top: 1px;
}

.c-alert__body { flex: 1; min-width: 0; }

.c-alert__title {
  font-size: var(--text-sm);
  font-weight: var(--fw-semibold);
  margin-bottom: 2px;
}

.c-alert__text { font-size: var(--text-sm); opacity: 0.9; line-height: 1.4; }

.c-alert__close {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  min-width: unset;
  min-height: unset;
  border-radius: var(--radius-sm);
  opacity: 0.6;
  transition: opacity var(--transition-fast);
}
.c-alert__close:hover { opacity: 1; }

/* ── Transition ────────────────────────────────────────────────── */
.c-alert-fade-enter-active { transition: all var(--transition-base); }
.c-alert-fade-leave-active { transition: all var(--transition-fast); }
.c-alert-fade-enter-from, .c-alert-fade-leave-to { opacity: 0; transform: translateY(-4px); }
</style>
