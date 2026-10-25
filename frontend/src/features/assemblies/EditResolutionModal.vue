<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Resolution } from '@/types/domain'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  meetingId: string
  meetingStatus: string
  resolution: Resolution | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'updated': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const form = ref({ description: '' })

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = { description: props.resolution?.description ?? '' }
  feedback.reset()
})

async function handleSubmit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.resolution) return
  saving.value = true
  feedback.reset()
  try {
    if (props.meetingStatus === 'draft') {
      await meetingsApi.updateResolution(props.meetingId, props.resolution.id, {
        number: props.resolution.number,
        title: props.resolution.title,
        description: form.value.description || undefined,
        vote_type: props.resolution.vote_type,
      })
    } else {
      await meetingsApi.patchResolution(props.meetingId, props.resolution.id, {
        description: form.value.description || undefined,
      })
    }
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
    :title="t('common.edit')"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-edit-resolution"
      @submit="handleSubmit"
    >
      <template v-if="meetingStatus === 'ongoing'">
        <p class="form-hint m-0 resolution-locked-notice">
          {{ t('meetings.resolutionDescriptionOnly') }}
        </p>
        <div class="flex flex-col gap-1">
          <span class="text-sm font-medium text-on-surface-variant">{{ t('meetings.resolutionLabel') }}</span>
          <span class="form-static">{{ resolution?.title }}</span>
        </div>
      </template>
      <CFormTextarea
        v-model="form.description"
        :label="t('common.description')"
        :placeholder="t('meetings.resolutionDescriptionPlaceholder')"
      />
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
          form="meeting-edit-resolution"
          :title="t('common.save')"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>
.resolution-locked-notice {
  grid-column: 1 / -1;
  padding: var(--space-2) var(--space-3);
  background: var(--color-surface-container-low);
  border-radius: var(--radius-lg);
}

.form-static {
  font-size: var(--text-sm);
  color: var(--color-on-surface);
  padding: var(--space-2) 0;
}
</style>
