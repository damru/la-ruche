<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePageLoader } from '@/composables/usePageLoader'
import { useFormatters } from '@/composables/useFormatters'
import { useAuthStore } from '@/stores/auth'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CCard from '@/components/molecules/CCard.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import JudicialProceedingFormModal from '@/features/registers/JudicialProceedingFormModal.vue'
import CDeleteConfirmModal from '@/components/organisms/CDeleteConfirmModal.vue'
import { judicialProceedingsApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import type { JudicialProceeding } from '@/types/domain'

const { t } = useI18n()
const { formatDate } = useFormatters()
const authStore = useAuthStore()

const buildingName = ref('')
const proceedings = ref<JudicialProceeding[]>([])

const canEdit = computed(() => authStore.isManager)

async function loadData() {
  const [building, listRes] = await Promise.all([
    useBuildingStore().fetch(),
    judicialProceedingsApi.list(),
  ])
  buildingName.value = building.name
  proceedings.value = listRes.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

const ongoingCount = computed(
  () => proceedings.value.filter(p => p.status !== 'closed').length,
)

const columns = computed(() => [
  { key: 'subject', label: t('registers.judicial.subject') },
  { key: 'proceeding_type', label: t('registers.judicial.type') },
  { key: 'parties', label: t('registers.judicial.parties') },
  { key: 'status', label: t('common.status') },
  { key: 'opened_at', label: t('registers.judicial.openedAt') },
  ...(canEdit.value ? [{ key: 'actions', label: '', numeric: true }] : []),
])

function statusVariant(status: string): 'default' | 'warn' | 'valid' {
  if (status === 'closed') return 'default'
  if (status === 'suspended') return 'warn'
  return 'valid'
}

const showModal = ref(false)
const editing = ref<JudicialProceeding | null>(null)

function openCreate() {
  editing.value = null
  showModal.value = true
}

function openEdit(p: JudicialProceeding) {
  editing.value = p
  showModal.value = true
}

const showDeleteModal = ref(false)
const deletingProceeding = ref<JudicialProceeding | null>(null)

function openDelete(p: JudicialProceeding) {
  deletingProceeding.value = p
  showDeleteModal.value = true
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight class="icon-xs" />
          <span>{{ t('registers.judicial.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canEdit"
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('registers.judicial.new')"
          @click="openCreate"
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
          <CEyebrow>{{ t('registers.eyebrow') }}</CEyebrow>
          <CHeading type="header">
            {{ t('registers.judicial.title') }}
          </CHeading>
          <CMetaItem variant="sub">
            {{ t('registers.judicial.summary', { count: proceedings.length, ongoing: ongoingCount }) }}
          </CMetaItem>
        </header>

        <CEmptyState
          v-if="proceedings.length === 0"
          :title="t('registers.judicial.empty')"
          size="page"
        >
          <template #icon>
            <CIconContract />
          </template>
          <template
            v-if="canEdit"
            #actions
          >
            <CBtn
              variant="primary"
              size="sm"
              :icon="CIconPlus"
              :title="t('registers.judicial.new')"
              @click="openCreate"
            />
          </template>
        </CEmptyState>

        <CCard
          v-else
          class="p-0!"
        >
          <CDataTable
            :columns="columns"
            :rows="proceedings"
            row-key="id"
          >
            <template #cell-subject="{ row }">
              <strong class="data-md">{{ row.subject }}</strong>
              <span
                v-if="row.case_reference"
                class="block text-xs text-on-surface-variant"
              >{{ row.case_reference }}</span>
            </template>
            <template #cell-proceeding_type="{ row }">
              {{ t(`registers.judicial.types.${row.proceeding_type}`) }}
            </template>
            <template #cell-parties="{ row }">
              <span class="text-xs">
                {{ row.claimant }}
                <span class="text-on-surface-variant"> c/ </span>
                {{ row.defendant }}
              </span>
            </template>
            <template #cell-status="{ row }">
              <CBadge :variant="statusVariant(row.status)">
                {{ t(`registers.judicial.statuses.${row.status}`) }}
              </CBadge>
            </template>
            <template #cell-opened_at="{ row }">
              {{ formatDate(row.opened_at) }}
              <span
                v-if="row.closed_at"
                class="block text-xs text-on-surface-variant"
              >→ {{ formatDate(row.closed_at) }}</span>
            </template>
            <template #cell-actions="{ row }">
              <div class="action-btns">
                <CBtn
                  variant="icon"
                  :icon="CIconEdit"
                  :title="t('common.edit')"
                  :aria-label="t('common.edit')"
                  @click="openEdit(row)"
                />
                <CBtn
                  variant="icon"
                  :icon="CIconTrash"
                  class="tone--err"
                  :title="t('common.delete')"
                  :aria-label="t('common.delete')"
                  @click="openDelete(row)"
                />
              </div>
            </template>
          </CDataTable>
        </CCard>
      </template>
    </div>

    <JudicialProceedingFormModal
      v-model:open="showModal"
      :proceeding="editing"
      @saved="reload"
    />

    <CDeleteConfirmModal
      v-model:open="showDeleteModal"
      :title="t('registers.judicial.deleteTitle')"
      :message="t('registers.judicial.deleteMessage', { subject: deletingProceeding?.subject ?? '' })"
      :success-message="t('registers.judicial.deleteSuccess')"
      :on-delete="() => judicialProceedingsApi.delete(deletingProceeding!.id).then(() => {})"
      @deleted="reload"
    />
  </div>
</template>
