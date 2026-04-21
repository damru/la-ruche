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
      class="c-btn__spinner"
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
</style>