<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { budgetApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'

const props = defineProps<{
  open: boolean
  budgetId: string
  line: { id: string; label: string } | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'deleted': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const deleting = ref(false)

watch(() => props.open, (open) => {
  if (!open) feedback.reset()
})

async function handleDelete() {
  if (!props.line) return
  deleting.value = true
  feedback.reset()
  try {
    await budgetApi.deleteLine(props.budgetId, props.line.id)
    emit('deleted')
    emit('update:open', false)
  } catch {
    feedback.setError(t('common.error'))
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('common.confirmDelete')"
    @update:open="$emit('update:open', $event)"
  >
    <p>{{ line?.label }}</p>
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
          :title="t('common.delete')"
          :loading="deleting"
          @click="handleDelete"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
