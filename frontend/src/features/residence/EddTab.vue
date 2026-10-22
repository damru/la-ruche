<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CEmptyState from '@/components/atoms/CEmptyState.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormChipInput from '@/components/atoms/CFormChipInput.vue'
import CFileDropzone from '@/components/molecules/CFileDropzone.vue'
import CLinkedDocumentsModal from '@/components/organisms/CLinkedDocumentsModal.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconClock from '@/components/atoms/CIconClock.svg'
import { eddApi, allocationKeysApi, unitsApi, documentsApi } from '@/api'
import { extractApiError } from '@/api/client'
import { extractFileExtension } from '@/utils/files'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import type { Unit, AllocationKey, EddReference, EddDiscrepancy, Document } from '@/types/domain'

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
  units.value = unitsRes.data
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
const selectedFiles = ref<File[]>([])

const form = ref<{
  label: string
  edd_date: string
  shares: { unit_id: string; unit_number: string; allocation_key_id: string; share_value: number; archived: boolean }[]
}>({ label: '', edd_date: '', shares: [] })

function openForm() {
  feedback.reset()
  selectedFiles.value = []
  const generalKey = keys.value.find(k => k.key_type === 'general')
  form.value = {
    label: '',
    edd_date: '',
    shares: generalKey
      ? units.value.map(u => ({ unit_id: u.id, unit_number: u.unit_number, allocation_key_id: generalKey.id, share_value: 0, archived: !u.active }))
      : [],
  }
  showForm.value = true
}

async function uploadEddDocument(file: File): Promise<string> {
  const extension = extractFileExtension(file.name)
  const mimeType = file.type || 'application/octet-stream'
  const uploadUrl = await documentsApi.getUploadUrl(extension, mimeType, file.size)
  await documentsApi.uploadToS3(file, uploadUrl.data.url, mimeType)
  const document = await documentsApi.confirmUpload({
    key: uploadUrl.data.key,
    bucket: uploadUrl.data.bucket,
    original_name: file.name,
    mime_type: mimeType,
    size_bytes: file.size,
    visible_roles: ['manager', 'board_member'],
    category: 'edd',
  })
  return document.data.id
}

async function saveReference() {
  if (!form.value.label.trim() || !form.value.edd_date) {
    feedback.setError(t('common.required'))
    return
  }
  feedback.reset()
  saving.value = true
  try {
    let documentId: string | null = null
    if (selectedFiles.value[0]) {
      documentId = await uploadEddDocument(selectedFiles.value[0])
    }
    await eddApi.createReference({
      label: form.value.label,
      edd_date: form.value.edd_date,
      document_id: documentId,
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

defineExpose({ openForm, showForm })

async function downloadDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch (e: unknown) {
    feedback.setError(toErrorMessage(e, t('settings.edd.downloadError')))
  }
}

// ── History modal ────────────────────────────────────────────────
const showHistory = ref(false)
const historyDocuments = computed<Document[]>(() =>
  references.value
    .filter(r => r.document_id)
    .map(r => ({
      id: r.document_id as string,
      original_name: `${r.label} — ${r.edd_date}`,
      mime_type: 'application/pdf',
      size_bytes: 0,
      visible_roles: [],
      created_at: r.created_at,
    })),
)

function openHistory() {
  showHistory.value = true
}

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
  <div>
    <CPageError
      v-if="error"
      :message="error"
      @retry="reload"
    />
    <CPageLoader v-else-if="loading" />
    <template v-else>
      <header class="mb-4">
        <CHeading type="section">
          {{ t('settings.edd.title') }}
        </CHeading>
        <p class="body-sm text-on-surface-variant">
          {{ t('settings.edd.subtitle') }}
        </p>
      </header>

      <div
        v-if="!showForm && references.length > 0"
        class="flex justify-end mb-3"
      >
        <CBtn
          size="sm"
          variant="ghost"
          :icon="CIconClock"
          :title="t('common.history')"
          @click="openHistory"
        />
      </div>

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
                  :title="t('common.newEddReference')"
                  @click="openForm"
                />
              </template>
            </CEmptyState>
          </CCard>
        </template>
        <template v-else>
          <CCard class="mb-4">
            <CHeading
              type="card"
              class="mb-2"
            >
              {{ t('settings.edd.currentReference') }}
            </CHeading>
            <div class="flex items-center justify-between gap-3">
              <p class="body-sm text-on-surface-variant">
                {{ latestReference?.label }} — {{ latestReference?.edd_date }}
              </p>
              <CBtn
                v-if="latestReference?.document_id"
                variant="ghost"
                size="sm"
                :icon="CIconDownload"
                :title="t('common.download')"
                @click="downloadDocument(latestReference.document_id as string)"
              />
            </div>
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
              class="flex flex-col gap-3"
            >
              <li
                v-for="(d, idx) in discrepancies"
                :key="idx"
                class="flex items-center gap-2 text-sm border-b border-outline-light pb-2 last:border-b-0 last:pb-0"
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
          {{ t('common.newEddReference') }}
        </CHeading>
        <CForm
          :status="feedback.status.value"
          :message="feedback.message.value"
          @submit="saveReference"
        >
          <CFormInput
            v-model="form.label"
            :label="t('common.label')"
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

          <div class="flex flex-col gap-1">
            <label class="text-sm font-semibold">{{ t('settings.edd.documentField') }}</label>
            <CFileDropzone
              v-model:files="selectedFiles"
              :multiple="false"
              accept="application/pdf"
              :hint="t('settings.edd.documentFieldHint')"
            />
          </div>

          <div class="pt-2">
            <h3 class="text-md font-semibold mb-3">
              {{ t('settings.edd.sharesField') }}
            </h3>
            <p
              v-if="form.shares.length === 0"
              class="text-on-surface-variant"
            >
              {{ t('settings.keys.noUnits') }}
            </p>
            <div
              v-else
              class="flex flex-wrap gap-1.5"
            >
              <div
                v-for="share in form.shares"
                :key="share.unit_id"
                class="flex items-center gap-1"
              >
                <CFormChipInput
                  v-model="share.share_value"
                  :label="share.unit_number"
                  type="number"
                  :step="1"
                  :min="0"
                />
                <CBadge
                  v-if="share.archived"
                  variant="default"
                >
                  {{ t('common.lotArchive') }}
                </CBadge>
              </div>
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

      <CLegalCard class="mt-8">
        <CLegalNotice>
          <CLegalDescription
            :title="t('settings.edd.legalCard.title')"
            :summary="t('settings.edd.legalCard.summary')"
          />
        </CLegalNotice>
      </CLegalCard>
    </template>

    <CLinkedDocumentsModal
      :open="showHistory"
      :title="t('common.history')"
      :documents="historyDocuments"
      :empty-message="t('settings.edd.historyEmpty')"
      @update:open="showHistory = $event"
      @download="downloadDocument"
    />
  </div>
</template>
