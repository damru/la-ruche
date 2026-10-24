<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormRow from '@/components/molecules/CFormRow.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { unitsApi, allocationKeysApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { Unit, UnitType } from '@/types/domain'
import { DEFAULT_UNIT_TYPE, UNIT_TYPES } from '@/utils/constants'

const props = defineProps<{
  open: boolean
  units: Unit[]
  unit?: Unit | null
  generalKeyId?: string | null
  generalKeyName?: string | null
  shareValue?: number
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
}>()

const { t } = useI18n()
const unitFeedback = useFormFeedback()
const savingUnit = ref(false)
const { schedule: scheduleUnitClose, forceExpire: forceUnitClose } = useAutoClose()

const editingUnit = ref<{
  id?: string
  unit_number: string
  unit_type: UnitType
  block: string
  staircase: string
  floor: string
  description: string
  general_shares: number | string
}>({
  unit_number: '', unit_type: DEFAULT_UNIT_TYPE, block: '', staircase: '', floor: '', description: '', general_shares: 0,
})

const unitTypeOptions = computed(() =>
  UNIT_TYPES.map(ut => ({ value: ut, label: t(`units.types.${ut}`) })),
)

watch(() => props.open, async (open) => {
  if (!open) { forceUnitClose(); return }
  unitFeedback.reset()
  if (props.unit) {
    editingUnit.value = {
      id: props.unit.id,
      unit_number: props.unit.unit_number,
      unit_type: props.unit.unit_type,
      block: props.unit.block ?? '',
      staircase: props.unit.staircase ?? '',
      floor: props.unit.floor ?? '',
      description: props.unit.description ?? '',
      general_shares: props.shareValue ?? 0,
    }
    if (props.generalKeyId) {
      try {
        const res = await allocationKeysApi.listShares(props.generalKeyId)
        const match = res.data.shares.find((s: { unit_id: string; share_value: number }) => s.unit_id === props.unit!.id)
        editingUnit.value.general_shares = match?.share_value ?? 0
      } catch { /* keep current value */ }
    }
  } else {
    editingUnit.value = { unit_number: '', unit_type: DEFAULT_UNIT_TYPE, block: '', staircase: '', floor: '', description: '', general_shares: 0 }
  }
})

async function saveUnit() {
  if (unitFeedback.status.value === 'success') { forceUnitClose(); return }
  if (!editingUnit.value.unit_number.trim()) {
    unitFeedback.setError(t('common.required'))
    return
  }
  savingUnit.value = true
  unitFeedback.reset()
  try {
    const data = {
      unit_number: editingUnit.value.unit_number,
      unit_type: editingUnit.value.unit_type,
      block: editingUnit.value.block || undefined,
      staircase: editingUnit.value.staircase || undefined,
      floor: editingUnit.value.floor || undefined,
      description: editingUnit.value.description || undefined,
    }
    let unitId = editingUnit.value.id
    if (unitId) {
      await unitsApi.update(unitId, data)
    } else {
      const res = await unitsApi.create(data)
      unitId = res.data.id
    }
    if (props.generalKeyId && unitId) {
      await allocationKeysApi.setShares(props.generalKeyId, [
        { unit_id: unitId, share_value: Number(editingUnit.value.general_shares) },
      ])
    }
    unitFeedback.setSuccess(t('common.saved'))
    emit('saved')
    scheduleUnitClose(() => { emit('update:open', false) })
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    const msg = (e as { response?: { data?: { message?: string } } })?.response?.data?.message ?? ''
    if (status === 409 && msg === 'UNIT_NUMBER_ARCHIVED') {
      unitFeedback.setError(t('residents.unitNumberArchived'))
    } else if (status === 409) {
      unitFeedback.setError(t('residents.unitNumberExists'))
    } else {
      unitFeedback.setError(t('common.error'))
    }
  } finally {
    savingUnit.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="editingUnit.id ? t('residents.editUnit') : t('residents.createUnit')"
    :closable="!savingUnit"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="unit-form"
      @submit="saveUnit"
    >
      <CFormRow :cols="2">
        <CFormInput
          v-model="editingUnit.unit_number"
          :label="t('units.number')"
          placeholder="A01"
          required
        />
        <CFormSelect
          v-model="editingUnit.unit_type"
          :label="t('common.type')"
          :options="unitTypeOptions"
        />
        <CFormInput
          v-model="editingUnit.block"
          :label="t('settings.tabs.building')"
        />
        <CFormInput
          v-model="editingUnit.staircase"
          :label="t('residents.staircaseLabel')"
        />
        <CFormInput
          v-model="editingUnit.floor"
          :label="t('residents.floorLabel')"
        />
        <CFormTextarea
          v-model="editingUnit.description"
          :label="t('common.description')"
          class="flex flex-col gap-1 col-span-full"
        />
      </CFormRow>

      <div class="pt-4">
        <h3 class="text-md font-semibold mb-3">
          {{ t('units.generalShares') }}
        </h3>
        <CFormInput
          v-model="editingUnit.general_shares"
          :label="generalKeyName ?? t('units.shares')"
          type="number"
          :step="1"
          :min="0"
          :hint="t('units.generalSharesHint')"
          class="max-w-60"
        />
      </div>
    </CForm>
    <template #footer>
      <CFormActions
        :status="unitFeedback.status.value"
        :message="unitFeedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          type="submit"
          form="unit-form"
          :title="t('common.save')"
          :loading="savingUnit"
          :closing="unitFeedback.status.value === 'success'"
          :disabled="savingUnit"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
