<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CFormDropzone from '@/components/molecules/CFormDropzone.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import { documentsApi, buildingApi } from '@/api'
import { toErrorMessage, usePageLoader } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { Document } from '@/types/domain'
import { buildDocumentDeleteMessage } from '@/utils/documents'
import { extractFileExtension } from '@/utils/files'

const { t } = useI18n()
const { formatDate, formatFileSize } = useFormatters()

const search = ref('')
const allDocuments = ref<Document[]>([])
const buildingName = ref('')

// ── Upload modal state ────────────────────────────────────────────
const showUpload = ref(false)
const uploading = ref(false)
const uploadError = ref('')
const uploadSuccess = ref(false)
const selectedFile = ref<File | null>(null)
const selectedRoles = ref<string[]>(['manager', 'board_member', 'co_owner'])
const { schedule: scheduleUploadClose, forceExpire: forceUploadClose } = useAutoClose()
watch(showUpload, (open) => { if (!open) forceUploadClose() })

const documents = computed(() => {
  const q = search.value.toLowerCase()
  if (!q) return allDocuments.value
  return allDocuments.value.filter(d => d.original_name.toLowerCase().includes(q))
})

async function loadData() {
  const [docsRes, buildingRes] = await Promise.all([
    documentsApi.list(),
    buildingApi.get(),
  ])
  allDocuments.value = docsRes.data
  buildingName.value = buildingRes.data.name
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

function documentReferences(document: Document): string[] {
  return [
    ...(document.linked_contract_names ?? []).map(name => t('documents.referenceContract', { name })),
    ...(document.linked_report_titles ?? []).map(title => t('documents.referenceReport', { title })),
    ...(document.linked_claim_titles ?? []).map(title => t('documents.referenceClaim', { title })),
  ]
}

function mimeLabel(mime: string): string {
  if (mime.includes('pdf')) return 'PDF'
  if (mime.includes('sheet') || mime.includes('excel')) return 'Excel'
  if (mime.includes('word')) return 'Word'
  return t('documents.fileTypeFallback')
}

const docColumns = computed(() => [
  { key: 'original_name', label: t('documents.columns.fileName') },
  { key: 'mime_type', label: t('documents.columns.type') },
  { key: 'visible_roles', label: t('documents.columns.visibility') },
  { key: 'references', label: t('documents.columns.references') },
  { key: 'size_bytes', label: t('documents.columns.size') },
  { key: 'created_at', label: t('documents.columns.addedDate') },
  { key: 'actions', label: '' },
])

function openUploadModal() {
  selectedFile.value = null
  uploadError.value = ''
  uploadSuccess.value = false
  selectedRoles.value = ['manager', 'board_member', 'co_owner']
  showUpload.value = true
}

function onDropzoneFiles(files: File[]) {
  selectedFile.value = files[0] ?? null
}

async function uploadFile() {
  if (uploadSuccess.value) { forceUploadClose(); return }
  if (!selectedFile.value) return
  uploading.value = true
  uploadError.value = ''

  try {
    const file = selectedFile.value
    const ext = extractFileExtension(file.name)

    // 1. Get presigned upload URL
    const urlRes = await documentsApi.getUploadUrl(ext, file.type, file.size)
    const { url, key, bucket } = urlRes.data

    // 2. Upload directly to S3
    await documentsApi.uploadToS3(file, url)

    // 3. Confirm upload in backend
    await documentsApi.confirmUpload({
      key,
      bucket,
      original_name: file.name,
      mime_type: file.type,
      size_bytes: file.size,
      visible_roles: selectedRoles.value,
    })

    uploadSuccess.value = true
    await loadData()
    scheduleUploadClose(() => { showUpload.value = false; uploadSuccess.value = false })
  } catch (e: unknown) {
    uploadError.value = toErrorMessage(e, t('documents.uploadFailed'))
  } finally {
    uploading.value = false
  }
}

async function downloadDoc(id: string) {
  try {
    const res = await documentsApi.getDownloadUrl(id)
    window.open(res.data.url, '_blank', 'noopener')
  } catch {
    error.value = t('common.error')
  }
}

// ── Delete document modal ─────────────────────────────────────────

const showDeleteDocModal = ref(false)
const docToDelete = ref<Document | null>(null)
const deletingDoc = ref(false)
const deleteDocFeedback = useFormFeedback()
const { schedule: scheduleDeleteDocClose, forceExpire: forceDeleteDocClose } = useAutoClose()
watch(showDeleteDocModal, (open) => { if (!open) forceDeleteDocClose() })

function openDeleteDocModal(doc: Document) {
  docToDelete.value = doc
  deleteDocFeedback.reset()
  showDeleteDocModal.value = true
}

async function confirmDeleteDoc() {
  if (deleteDocFeedback.status.value === 'success') { forceDeleteDocClose(); return }
  if (!docToDelete.value) return
  const docId = docToDelete.value.id
  deletingDoc.value = true
  deleteDocFeedback.reset()
  try {
    await documentsApi.delete(docId)
    allDocuments.value = allDocuments.value.filter(d => d.id !== docId)
    deleteDocFeedback.setSuccess(t('documents.deleteSuccess'))
    scheduleDeleteDocClose(() => { showDeleteDocModal.value = false })
  } catch (e: unknown) {
    deleteDocFeedback.setError(toErrorMessage(e))
  } finally {
    deletingDoc.value = false
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName || '...' }}
          <CIconChevronRight />
          <span>{{ t('documents.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('documents.import')"
          @click="openUploadModal"
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
            {{ t('page.eyebrows.documents') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('documents.title') }}
          </CHeading>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('documents.count', { count: allDocuments.length }) }}
          </p>
        </header>

        <CFormInput
          v-model="search"
          type="search"
          :placeholder="t('documents.searchPlaceholder')"
        >
          <template #prefix>
            <CIconSearch />
          </template>
        </CFormInput>

        <CCard class="p-0!">
          <CDataTable
            :columns="docColumns"
            :rows="documents"
            row-key="id"
            :empty-text="t('documents.empty')"
          >
            <template #cell-original_name="{ row }">
              <div class="doc-name-cell">
                <CIconFile
                  class="doc-icon"
                  aria-hidden="true"
                />
                <span class="doc-name">{{ row.original_name }}</span>
              </div>
            </template>
            <template #cell-mime_type="{ row }">
              <CBadge variant="default">
                {{ mimeLabel(row.mime_type) }}
              </CBadge>
            </template>
            <template #cell-visible_roles="{ row }">
              <div class="roles-list">
                <CBadge
                  v-for="role in row.visible_roles"
                  :key="role"
                  :variant="role"
                >
                  {{ t(`roles.${role}`) }}
                </CBadge>
              </div>
            </template>
            <template #cell-references="{ row }">
              <div
                v-if="documentReferences(row).length"
                class="doc-references"
              >
                <span
                  v-for="reference in documentReferences(row)"
                  :key="reference"
                  class="doc-reference"
                >
                  {{ reference }}
                </span>
              </div>
              <span
                v-else
                class="dim"
              >—</span>
            </template>
            <template #cell-size_bytes="{ row }">
              <span class="dim">{{ formatFileSize(row.size_bytes) }}</span>
            </template>
            <template #cell-created_at="{ row }">
              <span class="dim">{{ formatDate(row.created_at) }}</span>
            </template>
            <template #cell-actions="{ row }">
              <div class="action-btns">
                <CBtn
                  variant="icon"
                  :icon="CIconDownload"
                  :title="t('documents.downloadNewTab')"
                  :aria-label="t('documents.downloadNewTab')"
                  @click="downloadDoc(row.id)"
                >
              </CBtn>
                <CBtn
                  variant="icon"
                  :icon="CIconTrash"
                  class="tone--err"
                  :title="t('common.delete')"
                  :aria-label="t('common.delete')"
                  @click="openDeleteDocModal(row)"
                >
                </CBtn>
              </div>
            </template>
          </CDataTable>
        </CCard>
      </template>

      <!-- Upload modal -->
      <CModal
        :open="showUpload"
        :title="t('documents.upload')"
        @update:open="showUpload = $event"
      >
        <div class="upload-form">
          <CAlert
            v-if="uploadSuccess"
            variant="success"
            style="margin-bottom:12px"
          >
            {{ t('common.saved') }}
          </CAlert>
          <CAlert
            v-if="uploadError"
            variant="error"
            style="margin-bottom:12px"
          >
            {{ uploadError }}
          </CAlert>

          <CFormDropzone
            :uploading="uploading"
            @files="onDropzoneFiles"
            @error="uploadError = $event"
          />
          <p
            v-if="selectedFile"
            class="selected-file"
          >
            {{ selectedFile.name }} ({{ formatFileSize(selectedFile.size) }})
          </p>

          <div class="visibility-section">
            <label class="text-sm font-medium text-on-surface-variant">{{ t('documents.visiblePar') }}</label>
            <div class="role-checkboxes">
              <CFormCheckbox
                v-for="role in ['manager', 'board_member', 'co_owner', 'tenant']"
                :key="role"
                v-model="selectedRoles"
                :value="role"
                :label="t(`roles.${role}`)"
              />
            </div>
          </div>
        </div>

        <template #footer>
          <CBtn
            variant="ghost"
            :title="t('common.cancel')"
            @click="showUpload = false"
          />
          <CBtn
            variant="primary"
            :title="uploading ? t('common.saving') : t('common.upload')"
            :loading="uploading"
            :closing="uploadSuccess"
            :disabled="!selectedFile || uploading"
            @click="uploadFile"
          />
        </template>
      </CModal>

      <!-- Delete document modal -->
      <CModal
        :open="showDeleteDocModal"
        :title="t('documents.confirmDeleteTitle')"
        size="sm"
        @update:open="showDeleteDocModal = $event"
      >
        <p class="body-md">
          {{ buildDocumentDeleteMessage(docToDelete ?? undefined, t) }}
        </p>
        <template #footer>
          <CFormFeedback
            :status="deleteDocFeedback.status.value"
            :message="deleteDocFeedback.message.value"
          />
          <CBtn
            variant="ghost"
            :title="t('common.cancel')"
            @click="showDeleteDocModal = false"
          />
          <CBtn
            variant="danger"
            :title="t('common.deleteLabel')"
            :loading="deletingDoc"
            :closing="deleteDocFeedback.status.value === 'success'"
            @click="confirmDeleteDoc"
          />
        </template>
      </CModal>
    </div>
  </div>
</template>

<style scoped>

.doc-name-cell { display: flex; align-items: center; gap: 10px; }
.doc-icon { width: 18px; height: 18px; color: var(--color-primary); flex-shrink: 0; }
.doc-name { font-weight: 500; }
.dim { color: var(--color-on-surface-variant); }

.roles-list { display: flex; flex-wrap: wrap; gap: var(--space-1); }

.doc-references {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.doc-reference {
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

.action-btns { display: flex; gap: var(--space-1); }

/* Upload modal */
.upload-form { display: flex; flex-direction: column; gap: var(--space-4); }
.selected-file { color: var(--color-on-surface); font-weight: var(--fw-medium); }
.visibility-section { display: flex; flex-direction: column; gap: var(--space-2); }
.role-checkboxes { display: flex; flex-wrap: wrap; gap: var(--space-3); }

</style>
