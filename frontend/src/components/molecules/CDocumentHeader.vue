<script setup lang="ts">
// Molecule — CDocumentHeader
// Official document header for print views (EP-009-S09).
// Logo + app name on the left, document title centered, meta lines and
// edition date on the right. Styled for @media print (no background, thin
// bottom rule). Reused by every printable document view (logbook, future
// AG minutes, reminder letters…).

import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import CLogoMark from '@/components/atoms/CLogoMark.svg'
import { useFormatters } from '@/composables/useFormatters'

export interface DocumentHeaderMeta {
  label: string
  value: string
}

const props = defineProps<{
  title: string
  subtitle?: string
  meta?: DocumentHeaderMeta[]
  issuedOn?: Date | string
}>()

const { t } = useI18n()
const { formatDate } = useFormatters()

const issuedOnLabel = computed(() => {
  if (!props.issuedOn) return ''
  const value = props.issuedOn instanceof Date ? props.issuedOn.toISOString() : props.issuedOn
  return formatDate(value, 'long')
})
</script>

<template>
  <header class="document-header">
    <div class="document-header__brand">
      <CLogoMark
        class="document-header__logo"
        aria-hidden="true"
      />
      <span class="document-header__app-name">{{ t('nav.appName') }}</span>
    </div>
    <div class="document-header__title">
      <p class="document-header__title-main">
        {{ title }}
      </p>
      <p
        v-if="subtitle"
        class="document-header__title-sub"
      >
        {{ subtitle }}
      </p>
    </div>
    <div class="document-header__meta">
      <div
        v-for="item in meta"
        :key="item.label"
        class="document-header__meta-item"
      >
        <span class="document-header__meta-label">{{ item.label }}</span>
        <span class="document-header__meta-value">{{ item.value }}</span>
      </div>
      <p
        v-if="issuedOnLabel"
        class="document-header__issued"
      >
        {{ t('maintenance.print.editedOn', { date: issuedOnLabel }) }}
      </p>
    </div>
  </header>
</template>

<style scoped>
.document-header {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: start;
  gap: var(--space-4);
  padding-bottom: var(--space-4);
  margin-bottom: var(--space-6);
  border-bottom: 1px solid var(--color-outline-variant);
}

.document-header__brand {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.document-header__logo {
  width: var(--icon-lg);
  height: var(--icon-lg);
  flex-shrink: 0;
}

.document-header__app-name {
  font-family: var(--font-display, serif);
  font-weight: var(--fw-semibold);
  font-size: var(--text-md);
  color: var(--color-on-surface);
}

.document-header__title {
  text-align: center;
}

.document-header__title-main {
  margin: 0;
  font-family: var(--font-display, serif);
  font-weight: var(--fw-semibold);
  font-size: var(--text-xl);
  color: var(--color-on-surface);
}

.document-header__title-sub {
  margin: var(--space-1) 0 0;
  font-size: var(--text-sm);
  color: var(--color-on-surface-variant);
}

.document-header__meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: var(--space-1);
  text-align: right;
  font-size: var(--text-xs);
}

.document-header__meta-item {
  display: flex;
  flex-direction: column;
}

.document-header__meta-label {
  color: var(--color-on-surface-variant);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-size: var(--text-2xs, 0.625rem);
}

.document-header__meta-value {
  color: var(--color-on-surface);
}

.document-header__issued {
  margin: var(--space-2) 0 0;
  color: var(--color-on-surface-variant);
}

@media (max-width: 720px) {
  .document-header {
    grid-template-columns: 1fr;
    justify-items: center;
    text-align: center;
  }

  .document-header__meta {
    align-items: center;
    text-align: center;
  }
}

@media print {
  .document-header {
    border-bottom: 1px solid #000;
  }

  .document-header__app-name,
  .document-header__title-main,
  .document-header__meta-value {
    color: #000;
  }

  .document-header__title-sub,
  .document-header__meta-label,
  .document-header__issued {
    color: #333;
  }
}
</style>
