<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import { documentsApi, buildingApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import type { Document } from '@/types/domain'
import UploadDocumentModal from './UploadDocumentModal.vue'
import DeleteDocumentModal from './DeleteDocumentModal.vue'

const { t } = useI18n()
const { formatDate, formatFileSize } = useFormatters()

const search = ref('')
const allDocuments = ref<Document[]>([])
const buildingName = ref('')

const showUpload = ref(false)
const showDeleteDocModal = ref(false)
const docToDelete = ref<Document | null>(null)

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

async function downloadDoc(id: string) {
  try {
    const res = await documentsApi.getDownloadUrl(id)
    window.open(res.data.url, '_blank', 'noopener')
  } catch {
    error.value = t('common.error')
  }
}

function openDeleteDocModal(doc: Document) {
  docToDelete.value = doc
  showDeleteDocModal.value = true
}

function onDocumentDeleted(id: string) {
  allDocuments.value = allDocuments.value.filter(d => d.id !== id)
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
          @click="showUpload = true"
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
                />
                <CBtn
                  variant="icon"
                  :icon="CIconTrash"
                  class="tone--err"
                  :title="t('common.delete')"
                  :aria-label="t('common.delete')"
                  @click="openDeleteDocModal(row)"
                />
              </div>
            </template>
          </CDataTable>
        </CCard>
      </template>

      <UploadDocumentModal
        v-model:open="showUpload"
        @uploaded="loadData"
      />

      <DeleteDocumentModal
        v-model:open="showDeleteDocModal"
        :document="docToDelete"
        @deleted="onDocumentDeleted"
      />
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

</style>
