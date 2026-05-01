<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
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
const fileInputRef = ref<HTMLInputElement | null>(null)
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

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
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

function openUploadModal() {
  selectedFile.value = null
  uploadError.value = ''
  uploadSuccess.value = false
  selectedRoles.value = ['manager', 'board_member', 'co_owner']
  showUpload.value = true
}

function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files?.length) {
    selectedFile.value = input.files[0]
  }
}

function toggleRole(role: string) {
  const idx = selectedRoles.value.indexOf(role)
  if (idx >= 0) selectedRoles.value.splice(idx, 1)
  else selectedRoles.value.push(role)
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
    deleteDocFeedback.setError(toErrorMessage(e, t('common.error')))
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
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('documents.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          @click="openUploadModal"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('documents.import') }}</span>
        </CBtn>
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
        <div class="view-header">
          <div>
            <h1 class="display-md">
              {{ t('documents.title') }}
            </h1>
            <p
              class="body-md"
              style="color:var(--color-on-surface-variant)"
            >
              {{ t('documents.count', { count: allDocuments.length }) }}
            </p>
          </div>
        </div>

        <div class="search-bar">
          <CIconSearch class="search-icon" />
          <input
            v-model="search"
            type="search"
            :placeholder="t('documents.searchPlaceholder')"
            class="search-input"
          />
        </div>

        <CCard padding="none">
          <table class="data-table">
            <thead>
              <tr>
                <th>{{ t('documents.columns.fileName') }}</th>
                <th>{{ t('documents.columns.type') }}</th>
                <th>{{ t('documents.columns.visibility') }}</th>
                <th>{{ t('documents.columns.references') }}</th>
                <th>{{ t('documents.columns.size') }}</th>
                <th>{{ t('documents.columns.addedDate') }}</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="doc in documents"
                :key="doc.id"
              >
                <td>
                  <div class="doc-name-cell">
                    <CIconFile
                      class="doc-icon"
                      aria-hidden="true"
                    />
                    <span class="doc-name">{{ doc.original_name }}</span>
                  </div>
                </td>
                <td>
                  <CBadge variant="default">
                    {{ mimeLabel(doc.mime_type) }}
                  </CBadge>
                </td>
                <td>
                  <div class="roles-list">
                    <CBadge
                      v-for="role in doc.visible_roles"
                      :key="role"
                      :variant="role"
                    >
                      {{ t(`roles.${role}`) }}
                    </CBadge>
                  </div>
                </td>
                <td>
                  <div
                    v-if="documentReferences(doc).length"
                    class="doc-references"
                  >
                    <span
                      v-for="reference in documentReferences(doc)"
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
                </td>
                <td class="dim">
                  {{ formatFileSize(doc.size_bytes) }}
                </td>
                <td class="dim">
                  {{ formatDate(doc.created_at) }}
                </td>
                <td>
                  <div class="action-btns">
                    <button
                      class="icon-btn"
                      :title="t('documents.downloadNewTab')"
                      :aria-label="t('documents.downloadNewTab')"
                      @click="downloadDoc(doc.id)"
                    >
                      <CIconDownload
                        width="16"
                        height="16"
                      />
                    </button>
                    <button
                      class="icon-btn icon-btn--danger"
                      :title="t('common.delete')"
                      :aria-label="t('common.delete')"
                      @click="openDeleteDocModal(doc)"
                    >
                      <CIconTrash
                        width="16"
                        height="16"
                      />
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
          <div
            v-if="!documents.length"
            class="empty-state"
          >
            {{ t('documents.empty') }}
          </div>
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

          <div
            class="dropzone"
            @click="fileInputRef?.click()"
          >
            <input
              ref="fileInputRef"
              type="file"
              style="display:none"
              @change="onFileChange"
            />
            <span
              v-if="selectedFile"
              class="selected-file"
            >{{ selectedFile.name }} ({{ formatFileSize(selectedFile.size) }})</span>
            <span v-else>{{ t('documents.dropzone') }}</span>
            <p class="body-sm dim">
              {{ t('documents.maxSize') }}
            </p>
          </div>

          <div class="visibility-section">
            <label class="field-label">{{ t('documents.visiblePar') }}</label>
            <div class="role-checkboxes">
              <label
                v-for="role in ['manager', 'board_member', 'co_owner', 'tenant']"
                :key="role"
                class="checkbox-label"
              >
                <input
                  type="checkbox"
                  :checked="selectedRoles.includes(role)"
                  @change="toggleRole(role)"
                />
                {{ t(`roles.${role}`) }}
              </label>
            </div>
          </div>
        </div>

        <template #footer>
          <CBtn
            variant="ghost"
            @click="showUpload = false"
          >
            {{ t('common.cancel') }}
          </CBtn>
          <CBtn
            variant="primary"
            :loading="uploading"
            :closing="uploadSuccess"
            :disabled="!selectedFile || uploading"
            @click="uploadFile"
          >
            {{ uploading ? t('common.saving') : t('common.upload') }}
          </CBtn>
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
            @click="showDeleteDocModal = false"
          >
            {{ t('common.cancel') }}
          </CBtn>
          <CBtn
            variant="danger"
            :loading="deletingDoc"
            :closing="deleteDocFeedback.status.value === 'success'"
            @click="confirmDeleteDoc"
          >
            {{ t('common.deleteLabel') }}
          </CBtn>
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
.action-btns .icon-btn {
  min-width: 36px;
  min-height: 36px;
}

.empty-state { padding: var(--space-8); text-align: center; font-size: var(--text-md); color: var(--color-on-surface-variant); }

/* Upload modal */
.upload-form { display: flex; flex-direction: column; gap: var(--space-4); }
.dropzone {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: var(--space-2); padding: var(--space-8);
  border: 2px dashed var(--color-outline-variant);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: border-color var(--transition-fast);
  font-size: var(--text-md); color: var(--color-on-surface-variant);
}
.dropzone:hover { border-color: var(--color-primary); }
.selected-file { color: var(--color-on-surface); font-weight: var(--fw-medium); }
.visibility-section { display: flex; flex-direction: column; gap: var(--space-2); }
.role-checkboxes { display: flex; flex-wrap: wrap; gap: var(--space-3); }

@media (max-width: 640px) {
  .action-btns .icon-btn {
    min-width: 44px;
    min-height: 44px;
    padding: 0;
  }
}
</style>
