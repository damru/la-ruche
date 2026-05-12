<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import type { FeedbackStatus } from '@/composables/useFormFeedback'

withDefaults(defineProps<{
  open: boolean
  title: string
  description: string
  warning?: string
  loading?: boolean
  status?: FeedbackStatus
  message?: string
}>(), {
  warning: '',
  message: '',
  status: 'idle',
})

const emit = defineEmits<{
  'update:open': [value: boolean]
  unlink: []
  'unlink-and-delete': []
}>()

const { t } = useI18n()
</script>

<template>
  <CModal
    :open="open"
    size="sm"
    :closable="!loading"
    :title="title"
    @update:open="emit('update:open', $event)"
  >
    <div class="document-action-modal">
      <p class="body-xs">
        {{ description }}
      </p>
      <p
        v-if="warning"
        class="document-action-modal__warning"
      >
        {{ warning }}
      </p>
    </div>

    <template #footer>
      <CFormFeedback
        :status="status"
        :message="message"
      />
      <CBtn
        variant="ghost"
        :disabled="loading"
        @click="emit('update:open', false)"
      >
        {{ t('common.cancel') }}
      </CBtn>
      <CBtn
        variant="secondary"
        :disabled="loading"
        @click="emit('unlink')"
      >
        {{ t('documents.unlinkOnly') }}
      </CBtn>
      <CBtn
        variant="danger"
        :loading="loading"
        @click="emit('unlink-and-delete')"
      >
        {{ t('documents.unlinkAndDelete') }}
      </CBtn>
    </template>
  </CModal>
</template>

<style scoped>
.document-action-modal {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.document-action-modal__warning {
  margin: 0;
  color: var(--color-error);
}
</style>
