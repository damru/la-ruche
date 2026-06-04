<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
  meetingId: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'started': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const starting = ref(false)
const form = ref(defaultForm())

function defaultForm() {
  return {
    president_name: '',
    secretary_name: '',
    scrutineers: [] as Array<{ name: string }>,
  }
}

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = defaultForm()
  feedback.reset()
})

function addScrutineer() {
  form.value.scrutineers.push({ name: '' })
}

function removeScrutineer(index: number) {
  form.value.scrutineers.splice(index, 1)
}

async function handleSubmit() {
  if (!form.value.president_name.trim() || !form.value.secretary_name.trim()) {
    feedback.setError(t('meetings.start_modal.required'))
    return
  }
  starting.value = true
  feedback.reset()
  try {
    await meetingsApi.start(props.meetingId, {
      president_name: form.value.president_name.trim(),
      secretary_name: form.value.secretary_name.trim(),
      scrutineers: form.value.scrutineers
        .map(s => ({ name: s.name.trim() }))
        .filter(s => s.name.length > 0),
    })
    feedback.setSuccess(t('common.saved'))
    scheduleClose(() => {
      emit('update:open', false)
      emit('started')
    })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    starting.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('meetings.start_modal.title')"
    size="md"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-start"
      @submit="handleSubmit"
    >
      <p class="body-sm">
        {{ t('meetings.start_modal.lead') }}
      </p>
      <CFormInput
        v-model="form.president_name"
        :label="t('meetings.start_modal.president_label')"
        required
      />
      <CFormInput
        v-model="form.secretary_name"
        :label="t('meetings.start_modal.secretary_label')"
        required
      />
      <div class="flex flex-col gap-2">
        <span class="label-sm">{{ t('meetings.start_modal.scrutineers_label') }}</span>
        <div
          v-for="(scrutineer, index) in form.scrutineers"
          :key="index"
          class="flex gap-2 items-center"
        >
          <CFormInput
            v-model="scrutineer.name"
            :placeholder="t('meetings.start_modal.scrutineer_placeholder')"
            class="grow"
          />
          <CBtn
            variant="ghost"
            size="sm"
            type="button"
            :icon="CIconTrash"
            :title="t('common.delete')"
            @click="removeScrutineer(index)"
          />
        </div>
        <CBtn
          variant="ghost"
          size="sm"
          type="button"
          :icon="CIconPlus"
          :title="t('meetings.start_modal.add_scrutineer')"
          @click="addScrutineer"
        />
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
          form="meeting-start"
          :title="t('meetings.start')"
          :loading="starting"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
