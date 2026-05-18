<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import type { Document } from '@/types/domain'
import CBtn from '@/components/atoms/CBtn.vue'
import CModal from '@/components/molecules/CModal.vue'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconFile from '@/components/atoms/CIconFile.svg'

defineProps<{
  open: boolean
  title: string
  documents: Document[]
  loading?: boolean
  emptyMessage: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  download: [documentId: string]
}>()

const { t } = useI18n()
const { formatDate, formatFileSize } = useFormatters()
</script>

<template>
  <CModal
    :open="open"
    :title="title"
    size="md"
    @update:open="emit('update:open', $event)"
  >
    <div
      v-if="loading"
      class="linked-documents-modal__loading body-xs"
    >
      {{ t('common.loading') }}
    </div>

    <div
      v-else-if="documents.length"
      class="linked-documents-modal__list"
    >
      <div
        v-for="document in documents"
        :key="document.id"
        class="linked-documents-modal__row"
      >
        <div class="linked-documents-modal__meta">
          <div class="linked-documents-modal__name">
            <CIconFile />
            <span>{{ document.original_name }}</span>
          </div>
          <div class="linked-documents-modal__details">
            <span>{{ formatDate(document.created_at) }}</span>
            <span>•</span>
            <span>{{ formatFileSize(document.size_bytes) }}</span>
          </div>
        </div>
        <CBtn
          variant="ghost"
          size="sm"
          @click="emit('download', document.id)"
        >
          <CIconDownload />
          {{ t('common.download') }}
        </CBtn>
      </div>
    </div>

    <p
      v-else
      class="linked-documents-modal__empty body-xs"
    >
      {{ emptyMessage }}
    </p>

    <template #footer>
      <CBtn
        variant="ghost"
        @click="emit('update:open', false)"
      >
        {{ t('common.close') }}
      </CBtn>
    </template>
  </CModal>
</template>

<style scoped>
.linked-documents-modal__loading,
.linked-documents-modal__empty {
  color: var(--color-on-surface-variant);
}

.linked-documents-modal__list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.linked-documents-modal__row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
}

.linked-documents-modal__meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.linked-documents-modal__name {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
  font-weight: var(--fw-medium);
}

.linked-documents-modal__name svg {
  width: var(--icon-xs);
  height: var(--icon-xs);
  flex-shrink: 0;
}

.linked-documents-modal__name span {
  overflow: clip;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.linked-documents-modal__details {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

@media (max-width: 720px) {
  .linked-documents-modal__row {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
