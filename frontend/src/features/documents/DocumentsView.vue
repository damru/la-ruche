<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.vue'
import CIconDownload from '@/components/atoms/CIconDownload.vue'
import CIconSearch from '@/components/atoms/CIconSearch.vue'
import { mockDocuments, mockBuildingStats } from '@/mock/data'

const { t } = useI18n()
const { formatDate, formatFileSize } = useFormatters()

const search = ref('')

const documents = computed(() =>
  search.value
    ? mockDocuments.filter(d => d.original_name.toLowerCase().includes(search.value.toLowerCase()))
    : mockDocuments
)

function mimeIcon(mime: string): string {
  if (mime.includes('pdf')) return '📄'
  if (mime.includes('sheet') || mime.includes('excel')) return '📊'
  if (mime.includes('word')) return '📝'
  if (mime.includes('image')) return '🖼️'
  return '📎'
}

function mimeLabel(mime: string): string {
  if (mime.includes('pdf')) return 'PDF'
  if (mime.includes('sheet') || mime.includes('excel')) return 'Excel'
  if (mime.includes('word')) return 'Word'
  return 'Fichier'
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ mockBuildingStats.name }}
          <CIconChevronRight width="14" height="14"/>
          <span>{{ t('documents.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn variant="primary" size="sm">
          <CIconDownload style="width:16px;height:16px"/>
          Importer un document
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container">
    <CMockBanner />

    <div class="view-header">
      <div>
        <h1 class="display-md">{{ t('documents.title') }}</h1>
        <p class="body-md" style="color:var(--color-on-surface-variant)">
          {{ mockDocuments.length }} documents
        </p>
      </div>
    </div>

    <div class="search-bar">
      <CIconSearch class="search-icon"/>
      <input v-model="search" type="search" placeholder="Rechercher un document…" class="search-input" />
    </div>

    <CCard padding="none">
      <table class="doc-table">
        <thead>
          <tr>
            <th>Nom du fichier</th>
            <th>Type</th>
            <th>Visibilité</th>
            <th>Taille</th>
            <th>Date d'ajout</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="doc in documents" :key="doc.id">
            <td>
              <div class="doc-name-cell">
                <span class="doc-icon">{{ mimeIcon(doc.mime_type) }}</span>
                <span class="doc-name">{{ doc.original_name }}</span>
              </div>
            </td>
            <td><CBadge variant="default">{{ mimeLabel(doc.mime_type) }}</CBadge></td>
            <td>
              <div class="roles-list">
                <CBadge v-for="role in doc.visible_roles" :key="role" :variant="role">
                  {{ t(`roles.${role}`) }}
                </CBadge>
              </div>
            </td>
            <td class="dim">{{ formatFileSize(doc.size_bytes) }}</td>
            <td class="dim">{{ formatDate(doc.created_at) }}</td>
            <td>
              <button class="icon-btn" title="Télécharger">
                <CIconDownload/>
              </button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="!documents.length" class="empty-state">Aucun document trouvé.</div>
    </CCard>
  </div>
  </div>
</template>

<style scoped>
.view-container { padding: 40px; max-width: 1100px; display: flex; flex-direction: column; gap: 24px; }
.view-header { display: flex; align-items: flex-start; justify-content: space-between; }

.search-bar {
  display: flex; align-items: center; gap: 10px;
  padding: 10px 16px;
  background: var(--color-surface-container-lowest);
  border: 1.5px solid var(--color-outline-variant);
  border-radius: var(--radius-md);
}
.search-icon { width: 16px; height: 16px; color: var(--color-on-surface-variant); flex-shrink: 0; }
.search-input { flex: 1; background: none; border: none; outline: none; font-size: 14px; color: var(--color-on-surface); }

.doc-table { width: 100%; border-collapse: collapse; }
.doc-table th {
  padding: 10px 16px; text-align: left; font-size: 12px; font-weight: 600;
  text-transform: uppercase; letter-spacing: 0.05em;
  color: var(--color-on-surface-variant);
  border-bottom: 1px solid var(--color-outline-variant);
}
.doc-table td { padding: 10px 16px; font-size: 13px; border-bottom: 1px solid var(--color-surface-container-low); vertical-align: middle; }
.doc-table tr:last-child td { border-bottom: none; }

.doc-name-cell { display: flex; align-items: center; gap: 10px; }
.doc-icon { font-size: 18px; }
.doc-name { font-weight: 500; }
.dim { color: var(--color-on-surface-variant); }

.roles-list { display: flex; flex-wrap: wrap; gap: 4px; }

.icon-btn {
  background: none; border: none; cursor: pointer;
  color: var(--color-on-surface-variant); padding: 4px;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  display: flex;
}
.icon-btn:hover { background: var(--color-surface-container); color: var(--color-primary); }
.icon-btn svg { width: 16px; height: 16px; }

.empty-state { padding: 32px; text-align: center; font-size: 14px; color: var(--color-on-surface-variant); }
</style>

