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
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import EtatDateRequestFormModal from '@/features/registers/EtatDateRequestFormModal.vue'
import EtatDateSnapshotModal from '@/features/registers/EtatDateSnapshotModal.vue'
import { etatDateApi, unitsApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import type { EtatDateRequest, Unit } from '@/types/domain'
import { ETAT_DATE_STATUS_GENERATED, ETAT_DATE_STATUS_REQUESTED, ETAT_DATE_STATUS_SENT } from '@/utils/constants'
import { toErrorMessage } from '@/composables/usePageLoader'

const { t } = useI18n()
const { formatDate } = useFormatters()
const authStore = useAuthStore()

const buildingName = ref('')
const requests = ref<EtatDateRequest[]>([])
const units = ref<Unit[]>([])

const canEdit = computed(() => authStore.isManager)

async function loadData() {
  const [building, listRes, unitsRes] = await Promise.all([
    useBuildingStore().fetch(),
    etatDateApi.list(),
    unitsApi.list(),
  ])
  buildingName.value = building.name
  requests.value = listRes.data
  units.value = unitsRes.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

function unitNumber(unitId: string): string {
  return units.value.find(u => u.id === unitId)?.unit_number ?? '—'
}

function statusVariant(status: EtatDateRequest['status']): 'pending' | 'valid' | 'default' | 'err' {
  if (status === ETAT_DATE_STATUS_REQUESTED) return 'pending'
  if (status === ETAT_DATE_STATUS_GENERATED) return 'valid'
  if (status === ETAT_DATE_STATUS_SENT) return 'default'
  return 'err'
}

function isOverdue(request: EtatDateRequest): boolean {
  return request.status === ETAT_DATE_STATUS_REQUESTED && request.due_date < new Date().toISOString().slice(0, 10)
}

const columns = computed(() => [
  { key: 'unit', label: t('common.lot') },
  { key: 'requester_name', label: t('registers.etatDate.requesterName') },
  { key: 'due_date', label: t('registers.etatDate.dueDate') },
  { key: 'status', label: t('status') },
  ...(canEdit.value ? [{ key: 'actions', label: '', numeric: true }] : []),
])

const showRequestModal = ref(false)
const showSnapshotModal = ref(false)
const selectedRequest = ref<EtatDateRequest | null>(null)
const generatingId = ref<string | null>(null)
const generateError = ref('')

async function generate(request: EtatDateRequest) {
  generatingId.value = request.id
  generateError.value = ''
  try {
    const res = await etatDateApi.generate(request.id)
    const idx = requests.value.findIndex(r => r.id === request.id)
    if (idx !== -1) requests.value[idx] = res.data
  } catch (err) {
    generateError.value = toErrorMessage(err)
  } finally {
    generatingId.value = null
  }
}

function viewSnapshot(request: EtatDateRequest) {
  selectedRequest.value = request
  showSnapshotModal.value = true
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight class="icon-xs" />
          <span>{{ t('registers.etatDate.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canEdit"
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('registers.etatDate.new')"
          @click="showRequestModal = true"
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
            {{ t('registers.etatDate.title') }}
          </CHeading>
          <CMetaItem variant="sub">
            {{ t('registers.etatDate.summary', { count: requests.length }) }}
          </CMetaItem>
        </header>

        <p
          v-if="generateError"
          class="tone--err data-sm"
        >
          {{ generateError }}
        </p>

        <CEmptyState
          v-if="requests.length === 0"
          :title="t('registers.etatDate.empty')"
          size="page"
        >
          <template
            v-if="canEdit"
            #actions
          >
            <CBtn
              variant="primary"
              size="sm"
              :icon="CIconPlus"
              :title="t('registers.etatDate.new')"
              @click="showRequestModal = true"
            />
          </template>
        </CEmptyState>

        <CCard
          v-else
          class="p-0!"
        >
          <CDataTable
            :columns="columns"
            :rows="requests"
            row-key="id"
          >
            <template #cell-unit="{ row }">
              <strong class="data-md">{{ unitNumber(row.unit_id) }}</strong>
            </template>
            <template #cell-due_date="{ row }">
              {{ formatDate(row.due_date) }}
            </template>
            <template #cell-status="{ row }">
              <CBadge
                :variant="isOverdue(row) ? 'err' : statusVariant(row.status)"
                dot
              >
                {{ isOverdue(row) ? t('registers.etatDate.overdue') : t(`registers.etatDate.statuses.${row.status}`) }}
              </CBadge>
            </template>
            <template #cell-actions="{ row }">
              <div class="action-btns">
                <CBtn
                  v-if="row.status !== ETAT_DATE_STATUS_SENT"
                  variant="ghost"
                  size="sm"
                  :loading="generatingId === row.id"
                  :title="row.status === ETAT_DATE_STATUS_REQUESTED ? t('budget.generate') : t('registers.etatDate.regenerate')"
                  @click="generate(row)"
                />
                <CBtn
                  v-if="row.snapshot"
                  variant="ghost"
                  size="sm"
                  :title="t('registers.etatDate.view')"
                  @click="viewSnapshot(row)"
                />
              </div>
            </template>
          </CDataTable>
        </CCard>
      </template>
    </div>

    <EtatDateRequestFormModal
      v-model:open="showRequestModal"
      @saved="reload"
    />
    <EtatDateSnapshotModal
      v-model:open="showSnapshotModal"
      :request="selectedRequest"
    />
  </div>
</template>
