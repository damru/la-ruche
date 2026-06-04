<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { MeetingAttendanceEntry, MeetingAttendanceMode } from '@/types/domain'

const props = defineProps<{
  open: boolean
  meetingId: string
  entries: MeetingAttendanceEntry[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'updated': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const draft = ref<MeetingAttendanceEntry[]>([])

const attendanceModeOptions = computed(() => [
  { value: 'present', label: t('meetings.attendanceModes.present') },
  { value: 'proxy', label: t('meetings.attendanceModes.proxy') },
  { value: 'correspondence', label: t('meetings.attendanceModes.correspondence') },
  { value: 'absent', label: t('meetings.attendanceModes.absent') },
])

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  draft.value = props.entries.map(entry => ({ ...entry }))
  feedback.reset()
})

async function handleSubmit() {
  saving.value = true
  feedback.reset()
  try {
    await meetingsApi.saveAttendance(props.meetingId, draft.value)
    feedback.setSuccess(t('common.saved'))
    scheduleClose(() => {
      emit('update:open', false)
      emit('updated')
    })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('meetings.attendanceTitle')"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-attendance"
      @submit="handleSubmit"
    >
      <p class="modal-subtitle">
        {{ t('meetings.attendanceHint') }}
      </p>
      <div class="attendance-edit-list">
        <div
          v-for="entry in draft"
          :key="entry.unit_id"
          class="attendance-edit-row"
        >
          <div class="attendance-edit-row__identity">
            <strong>{{ entry.unit_number }}</strong>
            <span>{{ entry.owner_name || '—' }}</span>
          </div>
          <CFormSelect
            v-model="entry.mode"
            :options="attendanceModeOptions"
          />
          <CFormInput
            v-if="entry.mode === 'proxy'"
            v-model="entry.representative_name"
            :placeholder="t('meetings.attendanceRepresentative')"
          />
        </div>
      </div>
    </CForm>

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
          variant="primary"
          type="submit"
          form="meeting-attendance"
          :title="t('common.save')"
          :loading="saving"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>
.modal-subtitle {
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
  margin: 0;
}

.attendance-edit-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.attendance-edit-row {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(160px, 220px) minmax(0, 1fr);
  justify-content: space-between;
  gap: var(--space-3);
  align-items: center;
  padding: var(--space-3);
  border-radius: var(--radius-xl);
  background: var(--color-surface-container-low);
}

.attendance-edit-row__identity {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.attendance-edit-row__identity span {
  color: var(--color-on-surface-variant);
}

@media (max-width: 960px) {
  .attendance-edit-row {
    grid-template-columns: 1fr;
  }
}
</style>
