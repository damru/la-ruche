<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <div
    class="c-page-loader"
    role="status"
    :aria-label="t('common.loading')"
  >
    <span
      class="c-page-loader__spinner"
      aria-hidden="true"
    ></span>
    <span>{{ t('common.loading') }}</span>
  </div>
</template>

<style scoped>
.c-page-loader {
  min-height: 220px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  color: var(--color-on-surface-variant);
}

.c-page-loader__spinner {
  width: var(--icon-sm, 16px);
  height: var(--icon-sm, 16px);
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  flex-shrink: 0;
  opacity: 0.6;
}
</style>