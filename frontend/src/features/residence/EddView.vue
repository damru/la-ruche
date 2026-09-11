<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CEmptyState from '@/components/atoms/CEmptyState.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormChipInput from '@/components/atoms/CFormChipInput.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import { eddApi, allocationKeysApi, unitsApi } from '@/api'
import { extractApiError } from '@/api/client'
import { usePageLoader } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import type { Unit, AllocationKey, EddReference, EddDiscrepancy } from '@/types/domain'

const { t } = useI18n()

// ── Reference data (units × keys, for the entry form and label lookups) ──
const units = ref<Unit[]>([])
const keys = ref<AllocationKey[]>([])
const unitLabel = (id: string) => units.value.find(u => u.id === id)?.unit_number ?? id
const keyLabel = (id: string) => keys.value.find(k => k.id === id)?.name ?? id

// ── Report state ──────────────────────────────────────────────────
const references = ref<EddReference[]>([])
const discrepancies = ref<EddDiscrepancy[]>([])
const hasReference = ref(false)

async function loadAll() {
  const [unitsRes, keysRes, refsRes] = await Promise.all([
    unitsApi.list(),
    allocationKeysApi.list(),
    eddApi.listReferences(),
  ])
  units.value = unitsRes.data.filter(u => u.active)
  keys.value = keysRes.data
  references.value = refsRes.data

  try {
    const checkRes = await eddApi.check()
    discrepancies.value = checkRes.data.discrepancies
    hasReference.value = true
  } catch (e: unknown) {
    // Non-blocking: no EDD reference yet is an expected empty state, not a page error.
    if ((e as { response?: { status?: number } })?.response?.status === 404) {
      hasReference.value = false
      discrepancies.value = []
    } else {
      throw e
    }
  }
}

const { loading, error, reload } = usePageLoader(loadAll)
onMounted(reload)

// ── Entry form ────────────────────────────────────────────────────
const showForm = ref(false)
const feedback = useFormFeedback()
const saving = ref(false)

const form = ref<{
  label: string
  edd_date: string
  shares: { unit_id: string; unit_number: string; allocation_key_id: string; share_value: number }[]
}>({ label: '', edd_date: '', shares: [] })

function openForm() {
  feedback.reset()
  const generalKey = keys.value.find(k => k.key_type === 'general')
  form.value = {
    label: '',
    edd_date: '',
    shares: generalKey
      ? units.value.map(u => ({ unit_id: u.id, unit_number: u.unit_number, allocation_key_id: generalKey.id, share_value: 0 }))
      : [],
  }
  showForm.value = true
}

async function saveReference() {
  if (!form.value.label.trim() || !form.value.edd_date) {
    feedback.setError(t('common.required'))
    return
  }
  feedback.reset()
  saving.value = true
  try {
    await eddApi.createReference({
      label: form.value.label,
      edd_date: form.value.edd_date,
      shares: form.value.shares
        .filter(s => s.share_value > 0)
        .map(s => ({ unit_id: s.unit_id, allocation_key_id: s.allocation_key_id, share_value: s.share_value })),
    })
    showForm.value = false
    await reload()
  } catch (e: unknown) {
    feedback.setError(t(extractApiError(e)))
  } finally {
    saving.value = false
  }
}

const latestReference = computed(() => references.value[0])

function discrepancyLabel(d: EddDiscrepancy): string {
  switch (d.kind) {
    case 'missing_in_edd': return t('settings.edd.discrepancy.missingInEdd', { unit: unitLabel(d.unit_id), key: keyLabel(d.allocation_key_id) })
    case 'missing_in_config': return t('settings.edd.discrepancy.missingInConfig', { unit: unitLabel(d.unit_id), key: keyLabel(d.allocation_key_id) })
    case 'value_mismatch': return t('settings.edd.discrepancy.valueMismatch', { unit: unitLabel(d.unit_id), key: keyLabel(d.allocation_key_id), expected: d.expected, actual: d.actual })
    case 'total_mismatch': return t('settings.edd.discrepancy.totalMismatch', { key: keyLabel(d.allocation_key_id), expected: d.expected, actual: d.actual })
  }
}

function discrepancyVariant(d: EddDiscrepancy): 'warn' | 'err' {
  return d.kind === 'total_mismatch' ? 'err' : 'warn'
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          <span>{{ t('nav.groups.residence') }}</span>
          <CIconChevronRight />
          <span>{{ t('nav.residence-edd') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          size="sm"
          :icon="CIconPlus"
          :title="t('settings.edd.newReference')"
          @click="openForm"
        />
      </template>
    </CTopbar>

    <div class="view-container">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else>
        <header class="view-header">
          <CEyebrow>
            {{ t('page.eyebrows.edd') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('settings.edd.title') }}
          </CHeading>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('settings.edd.subtitle') }}
          </p>
        </header>

        <CAlert variant="info">
          {{ t('settings.edd.nonBlockingNotice') }}
        </CAlert>

        <template v-if="!showForm">
          <template v-if="!hasReference">
            <CCard>
              <CEmptyState
                :title="t('settings.edd.empty')"
                :description="t('settings.edd.emptyDescription')"
                size="page"
              >
                <template #actions>
                  <CBtn
                    variant="primary"
                    :icon="CIconPlus"
                    :title="t('settings.edd.newReference')"
                    @click="openForm"
                  />
                </template>
              </CEmptyState>
            </CCard>
          </template>
          <template v-else>
            <CCard>
              <CHeading
                type="card"
                class="mb-2"
              >
                {{ t('settings.edd.currentReference') }}
              </CHeading>
              <p class="body-sm text-on-surface-variant">
                {{ latestReference?.label }} — {{ latestReference?.edd_date }}
              </p>
            </CCard>

            <CCard>
              <CHeading
                type="card"
                class="mb-4"
              >
                {{ t('settings.edd.reportTitle') }}
              </CHeading>

              <CEmptyState
                v-if="discrepancies.length === 0"
                :title="t('settings.edd.reportEmpty')"
                size="compact"
              />
              <ul
                v-else
                class="discrepancy-list"
              >
                <li
                  v-for="(d, idx) in discrepancies"
                  :key="idx"
                  class="discrepancy-item"
                >
                  <CBadge :variant="discrepancyVariant(d)">
                    {{ t(`settings.edd.discrepancy.kind.${d.kind}`) }}
                  </CBadge>
                  <span>{{ discrepancyLabel(d) }}</span>
                </li>
              </ul>
            </CCard>
          </template>
        </template>

        <CCard v-else>
          <CHeading
            type="card"
            class="mb-4"
          >
            {{ t('settings.edd.newReference') }}
          </CHeading>
          <CForm
            :status="feedback.status.value"
            :message="feedback.message.value"
            @submit="saveReference"
          >
            <CFormInput
              v-model="form.label"
              :label="t('settings.edd.labelField')"
              required
              class="flex flex-col gap-1"
            />
            <CFormInput
              v-model="form.edd_date"
              :label="t('settings.edd.dateField')"
              type="date"
              required
              class="flex flex-col gap-1"
            />

            <div class="shares-section">
              <h3 class="text-md font-semibold mb-3">
                {{ t('settings.edd.sharesField') }}
              </h3>
              <p
                v-if="form.shares.length === 0"
                class="dim"
              >
                {{ t('settings.keys.noUnits') }}
              </p>
              <div
                v-else
                class="c-chips"
              >
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
            </div>

            <template #actions>
              <CBtn
                variant="ghost"
                :title="t('common.cancel')"
                type="button"
                @click="showForm = false"
              />
              <CBtn
                variant="primary"
                type="submit"
                :title="t('common.save')"
                :loading="saving"
                :disabled="saving"
              />
            </template>
          </CForm>
        </CCard>
      </template>
    </div>
  </div>
</template>

<style scoped>
.dim { color: var(--color-on-surface-variant); }
.shares-section { padding-top: var(--space-2); }
.c-chips { display: flex; flex-wrap: wrap; gap: 6px; }
.discrepancy-list { display: flex; flex-direction: column; gap: var(--space-3); }
.discrepancy-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-sm);
  border-bottom: 1px solid var(--color-outline-light);
  padding-bottom: var(--space-2);
}
.discrepancy-item:last-child { border-bottom: none; padding-bottom: 0; }
</style>
