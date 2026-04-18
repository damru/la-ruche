<script setup lang="ts">
// EventDetailModal — view a manual calendar event (EP-014-S03, AC-CAL-005-D/E).
// Opened via ?event={id} from the calendar feed. Edit/Delete only for
// Manager/BoardMember (server guard is the real gate; this is UI defense in depth).

import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import { calendarEventsApi, documentsApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useFormatters } from '@/composables/useFormatters'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Document, ManualCalendarEvent } from '@/types/domain'

const props = defineProps<{
  open: boolean
  eventId: string | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'edit': [event: ManualCalendarEvent]
  'deleted': []
}>()

const { t } = useI18n()
const { formatDate, formatFileSize } = useFormatters()
const auth = useAuthStore()
const feedback = useFormFeedback()

const loading = ref(false)
const deleting = ref(false)
const event = ref<ManualCalendarEvent | null>(null)
const documents = ref<Document[]>([])

const canManage = auth.hasAnyRole(['manager', 'board_member'])

watch(() => props.open, async (open) => {
  if (!open || !props.eventId) return
  feedback.reset()
  loading.value = true
  event.value = null
  documents.value = []
  try {
    const [eventRes, docsRes] = await Promise.all([
      calendarEventsApi.get(props.eventId),
      calendarEventsApi.documents(props.eventId),
    ])
    event.value = eventRes.data
    documents.value = docsRes.data
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    loading.value = false
  }
})

async function downloadDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch {
    // download errors are non-critical in modal context
  }
}

async function confirmDelete() {
  if (!event.value) return
  deleting.value = true
  feedback.reset()
  try {
    await calendarEventsApi.remove(event.value.id)
    emit('deleted')
    emit('update:open', false)
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="event?.title ?? t('calendar.event.detailTitle')"
    size="md"
    @update:open="emit('update:open', $event)"
  >
    <CPageLoader v-if="loading" />
    <div
      v-else-if="event"
      class="flex flex-col gap-3"
    >
      <div class="flex items-center gap-2">
        <span class="body-sm">
          {{ formatDate(event.event_date) }}
          <template v-if="event.event_time">· {{ event.event_time.slice(0, 5) }}</template>
        </span>
        <CBadge
          v-if="event.category"
          variant="default"
        >
          {{ t(`calendar.event.categories.${event.category}`) }}
        </CBadge>
      </div>
      <p
        v-if="event.description"
        class="body-sm text-on-surface-variant"
      >
        {{ event.description }}
      </p>

      <div
        v-if="documents.length"
        class="flex flex-col gap-2"
      >
        <span class="text-xs text-on-surface-variant">{{ t('calendar.event.fields.documents') }}</span>
        <div
          v-for="document in documents"
          :key="document.id"
          class="flex items-center justify-between gap-3"
        >
          <div class="flex flex-col min-w-0">
            <span class="body-sm">{{ document.original_name }}</span>
            <span class="text-xs text-on-surface-variant">{{ formatFileSize(document.size_bytes) }}</span>
          </div>
          <CBtn
            variant="ghost"
            size="sm"
            :icon="CIconDownload"
            :title="t('common.download')"
            @click="downloadDocument(document.id)"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <CFormActions
        :status="feedback.status.value"
        :message="feedback.message.value"
      >
        <CBtn
          v-if="canManage && event"
          variant="danger"
          :icon="CIconTrash"
          :title="t('common.delete')"
          :loading="deleting"
          class="mr-auto"
          @click="confirmDelete"
        />
        <CBtn
          variant="ghost"
          :title="t('common.close')"
          @click="emit('update:open', false)"
        />
        <CBtn
          v-if="canManage && event"
          variant="primary"
          :title="t('common.edit')"
          @click="emit('edit', event)"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
