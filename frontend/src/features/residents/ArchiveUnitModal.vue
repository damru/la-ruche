<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { unitsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { Unit } from '@/types/domain'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  unit: Unit | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'archived': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const archiveFeedback = useFormFeedback()
const archivingUnit = ref(false)
const { schedule: scheduleArchiveClose, forceExpire: forceArchiveClose } = useAutoClose()

watch(() => props.open, (open) => {
  if (!open) { forceArchiveClose(); return }
  archiveFeedback.reset()
})

async function confirmArchiveUnit() {
  if (archiveFeedback.status.value === 'success') { forceArchiveClose(); return }
  if (!props.unit) return
  archivingUnit.value = true
  archiveFeedback.reset()
  try {
    await unitsApi.archive(props.unit.id)
    archiveFeedback.setSuccess(t('settings.edd.archivedUnit'))
    emit('archived')
    scheduleArchiveClose(() => { emit('update:open', false) })
  } catch {
    archiveFeedback.setError(t('common.error'))
  } finally {
    archivingUnit.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('residents.confirmArchiveTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('residents.confirmArchiveMessage', { number: unit?.unit_number ?? '' }) }}
    </p>
    <template #footer>
      <CFormActions
        :status="archiveFeedback.status.value"
        :message="archiveFeedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="danger"
          :title="t('common.archiveLabel')"
          :loading="archivingUnit"
          :closing="archiveFeedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          @click="confirmArchiveUnit"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
