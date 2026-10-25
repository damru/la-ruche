<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import { documentsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { buildDocumentDeleteMessage } from '@/utils/documents'
import type { Document } from '@/types/domain'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  document: Document | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'deleted': [id: string]
}>()

const { t } = useI18n()
const ui = useUiStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const deleting = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
})

async function confirmDelete() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.document) return
  const docId = props.document.id
  deleting.value = true
  feedback.reset()
  try {
    await documentsApi.delete(docId)
    feedback.setSuccess(t('documents.deleteSuccess'))
    emit('deleted', docId)
    scheduleClose(() => { emit('update:open', false) })
  } catch (e: unknown) {
    feedback.setError(toErrorMessage(e))
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('documents.confirmDeleteTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ buildDocumentDeleteMessage(document ?? undefined, t) }}
    </p>
    <template #footer>
      <CFormActions
        :status="feedback.status.value"
        :message="feedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="danger"
          :title="t('common.deleteLabel')"
          :loading="deleting"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          @click="confirmDelete"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
