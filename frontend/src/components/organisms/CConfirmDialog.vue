<script setup lang="ts">
// Organism — CConfirmDialog
// Singleton confirm dialog driven by the useConfirm composable.
// Mount once in App.vue — it reacts to any confirm() call app-wide.

import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { useConfirm } from '@/composables/useConfirm'

const { t } = useI18n()
const { state, respond } = useConfirm()
</script>

<template>
  <CModal
    :open="state !== null"
    :title="state?.title ?? ''"
    size="sm"
    :stack="20"
    :closable="false"
    @update:open="!$event && respond(false)"
  >
    <p class="body-md confirm-message">
      {{ state?.message }}
    </p>

    <template #footer>
      <CBtn
        variant="ghost"
        @click="respond(false)"
      >
        {{ t('common.cancel') }}
      </CBtn>
      <CBtn
        :variant="state?.variant === 'primary' ? 'primary' : 'danger'"
        @click="respond(true)"
      >
        {{ state?.confirmLabel ?? t('common.confirm') }}
      </CBtn>
    </template>
  </CModal>
</template>

<style scoped>
.confirm-message {
  color: var(--color-on-surface-variant);
  padding: var(--space-2) 0 var(--space-4);
}
</style>
