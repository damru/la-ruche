<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import { consultationsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)

const defaultForm = () => ({ title: '', description: '', choices: ['', ''] })
const form = ref(defaultForm())

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = defaultForm()
  feedback.reset()
})

function addChoice() { form.value.choices.push('') }
function removeChoice(idx: number) {
  if (form.value.choices.length > 2) form.value.choices.splice(idx, 1)
}

async function handleSubmit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!form.value.title.trim() || form.value.choices.filter(c => c.trim()).length < 2) return
  saving.value = true
  feedback.reset()
  try {
    await consultationsApi.create({
      title: form.value.title,
      description: form.value.description || undefined,
      visible_to_roles: ['manager', 'board_member', 'co_owner', 'tenant'],
      choices: form.value.choices.filter(c => c.trim()),
    })
    feedback.setSuccess(t('common.saved'))
    emit('created')
    scheduleClose(() => { emit('update:open', false) })
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
    :title="t('consultations.create')"
    size="lg"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="consultation-create"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="form.title"
        :label="t('consultations.fields.title')"
        required
      />
      <CFormTextarea
        v-model="form.description"
        :label="t('consultations.fields.description')"
        :rows="2"
      />
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('consultations.fields.choices') }} *</label>
        <div
          v-for="(_, idx) in form.choices"
          :key="idx"
          class="choice-input-row"
        >
          <CFormInput
            v-model="form.choices[idx]"
            :placeholder="`Option ${idx + 1}`"
            class="grow"
          />
          <CBtn
            v-if="form.choices.length > 2"
            type="button"
            variant="icon"
            :icon="CIconTrash"
            class="tone--err"
            @click="removeChoice(idx)"
          />
        </div>
        <CBtn
          variant="ghost"
          size="sm"
          type="button"
          :icon="CIconPlus"
          :title="t('consultations.fields.addChoice')"
          @click="addChoice"
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
          form="consultation-create"
          :title="t('common.create')"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>
.choice-input-row { display: flex; align-items: center; gap: 8px; }
.choice-input-row .form-input { flex: 1; }
</style>
