<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormChipInput from '@/components/atoms/CFormChipInput.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconSpinner from '@/components/atoms/CIconSpinner.svg'
import { allocationKeysApi, unitsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { AllocationKey } from '@/types/domain'

const props = defineProps<{
  open: boolean
  keyToEdit: AllocationKey | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': [key: AllocationKey, balanceDelta: number | null]
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const loading = ref(false)
const saving = ref(false)

const form = ref<{
  name: string
  description: string
  justification: string
  shares: { unit_id: string; unit_number: string; share_value: number }[]
  sharesTotal: number
}>({ name: '', description: '', justification: '', shares: [], sharesTotal: 0 })

function sortSharesByUnit(shares: { unit_id: string; unit_number: string; share_value: number }[]) {
  return [...shares].sort((a, b) =>
    a.unit_number.localeCompare(b.unit_number, undefined, { numeric: true }),
  )
}

const sharesSum = computed(() =>
  form.value.shares.reduce((acc, s) => acc + (s.share_value || 0), 0),
)

const isSumMismatch = computed(() => {
  const total = form.value.sharesTotal
  return total > 0 && sharesSum.value !== total
})

watch(() => props.open, async (open) => {
  if (!open) {
    forceClose()
    return
  }
  feedback.reset()
  form.value = { name: '', description: '', justification: '', shares: [], sharesTotal: 0 }
  loading.value = true
  try {
    if (props.keyToEdit) {
      form.value.name = props.keyToEdit.name
      form.value.description = props.keyToEdit.description ?? ''
      const res = await allocationKeysApi.listShares(props.keyToEdit.id)
      form.value.shares = sortSharesByUnit(res.data.shares.map(s => ({ ...s })))
      form.value.sharesTotal = res.data.shares_total
    } else {
      const res = await unitsApi.list()
      form.value.shares = sortSharesByUnit(
        res.data
          .filter(u => u.active)
          .map(u => ({ unit_id: u.id, unit_number: u.unit_number, share_value: 0 })),
      )
    }
  } catch { /* non-blocking — shares section will show empty state */ }
  finally { loading.value = false }
})

async function save() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!form.value.name.trim()) {
    feedback.setError(t('common.required'))
    return
  }
  feedback.reset()
  saving.value = true
  try {
    let savedKey: AllocationKey
    let keyId: string

    if (props.keyToEdit) {
      const res = await allocationKeysApi.update(props.keyToEdit.id, {
        name: form.value.name,
        description: form.value.description || undefined,
        justification: form.value.justification || undefined,
      })
      savedKey = res.data
      keyId = res.data.id
    } else {
      const res = await allocationKeysApi.create({
        name: form.value.name,
        description: form.value.description || undefined,
        justification: form.value.justification || undefined,
      })
      savedKey = res.data
      keyId = res.data.id
    }

    if (form.value.shares.length > 0) {
      await allocationKeysApi.setShares(keyId, form.value.shares.map(s => ({
        unit_id: s.unit_id,
        share_value: Number(s.share_value),
      })), form.value.justification || undefined)
    }

    let balanceDelta: number | null = null
    try {
      const bal = await allocationKeysApi.balance(keyId)
      balanceDelta = bal.data.delta
    } catch { /* non-blocking */ }

    feedback.setSuccess(t('common.saved'))
    emit('saved', savedKey, balanceDelta)
    scheduleClose(() => { emit('update:open', false) })
  } catch (e: unknown) {
    feedback.setError((e as { response?: { data?: { message?: string } } })?.response?.data?.message ?? t('common.error'))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    size="lg"
    :closable="!saving"
    :title="keyToEdit ? t('settings.keys.editKey') : t('settings.keys.newKey')"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="key-form"
      @submit="save"
    >
      <div class="flex flex-col gap-4">
        <CFormInput
          v-model="form.name"
          :label="t('lastName')"
          :placeholder="t('settings.keys.namePlaceholder')"
          required
          class="flex flex-col gap-1 col-span-full"
        />
        <CFormTextarea
          v-model="form.description"
          :label="t('common.description')"
          :placeholder="t('settings.keys.descriptionPlaceholder')"
          class="flex flex-col gap-1 col-span-full"
        />
        <CFormTextarea
          v-model="form.justification"
          :label="t('settings.keys.justificationLabel')"
          :placeholder="t('settings.keys.justificationPlaceholder')"
          :maxlength="500"
          class="flex flex-col gap-1 col-span-full"
        />
      </div>

      <div class="shares-section">
        <h3 class="text-md font-semibold mb-3">
          {{ t('common.shares') }}
        </h3>

        <div
          v-if="loading"
          class="shares-loading"
        >
          <CIconSpinner class="spin icon-md" />
          <span>{{ t('common.loading') }}</span>
        </div>
        <template v-else>
          <p
            v-if="form.shares.length === 0"
            class="dim"
          >
            {{ t('settings.keys.noUnits') }}
          </p>
          <template v-else>
            <div class="c-chips">
              <CFormChipInput
                v-for="share in form.shares"
                :key="share.unit_id"
                v-model="share.share_value"
                :label="share.unit_number"
                type="number"
                :step="1"
                :min="0"
              />
            </div>
            <div class="shares-summary">
              <span>{{ t('common.total') }} :</span>
              <span
                class="font-semibold"
                :class="{ 'sum-mismatch': isSumMismatch }"
              >
                {{ sharesSum }}
              </span>
              <span
                v-if="form.sharesTotal > 0"
                class="dim"
              >/ {{ form.sharesTotal }}</span>
            </div>
            <CAlert
              v-if="isSumMismatch"
              variant="warning"
              class="mt-3"
            >
              {{ t('settings.keys.sumMismatch') }}
            </CAlert>
          </template>
        </template>
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
          @click="emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          type="submit"
          form="key-form"
          :title="t('common.save')"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
          :disabled="saving || loading"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>
.shares-section { padding-top: var(--space-4); }
.shares-loading { display: flex; align-items: center; gap: var(--space-2); color: var(--color-on-surface-variant); font-size: var(--text-sm); }
.c-chips { display: flex; flex-wrap: wrap; gap: 6px; }
.shares-summary {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--text-sm);
  color: var(--color-on-surface-variant);
}
.sum-mismatch { color: var(--color-error); }
.dim { color: var(--color-on-surface-variant); }
</style>
