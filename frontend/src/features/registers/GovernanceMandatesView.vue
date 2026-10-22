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
import GovernanceMandateFormModal from '@/features/registers/GovernanceMandateFormModal.vue'
import GovernanceMandateDeleteModal from '@/features/registers/GovernanceMandateDeleteModal.vue'
import { governanceApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import type { GovernanceMandate } from '@/types/domain'

const { t } = useI18n()
const { formatDate } = useFormatters()
const authStore = useAuthStore()

const buildingName = ref('')
const mandates = ref<GovernanceMandate[]>([])

const canEdit = computed(() => authStore.isManager)

async function loadData() {
  const [building, listRes] = await Promise.all([
    useBuildingStore().fetch(),
    governanceApi.list(),
  ])
  buildingName.value = building.name
  mandates.value = listRes.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

const activeCount = computed(
  () => mandates.value.filter(m => m.end_date === null).length,
)

const columns = computed(() => [
  { key: 'holder_name', label: t('registers.governance.holder') },
  { key: 'role', label: t('registers.governance.role') },
  { key: 'start_date', label: t('registers.governance.startDate') },
  { key: 'period', label: t('registers.governance.endDate') },
  ...(canEdit.value ? [{ key: 'actions', label: '', numeric: true }] : []),
])

function roleVariant(role: string): 'valid' | 'accent' | 'default' {
  if (role === 'syndic') return 'valid'
  if (role === 'council_president') return 'accent'
  return 'default'
}

const showModal = ref(false)
const editing = ref<GovernanceMandate | null>(null)

function openCreate() {
  editing.value = null
  showModal.value = true
}

function openEdit(m: GovernanceMandate) {
  editing.value = m
  showModal.value = true
}

const showDeleteModal = ref(false)
const deletingMandate = ref<GovernanceMandate | null>(null)

function openDelete(m: GovernanceMandate) {
  deletingMandate.value = m
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
          <span>{{ t('registers.governance.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canEdit"
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('registers.governance.new')"
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
            {{ t('registers.governance.title') }}
          </CHeading>
          <CMetaItem variant="sub">
            {{ t('registers.governance.summary', { count: mandates.length, active: activeCount }) }}
          </CMetaItem>
        </header>

        <CEmptyState
          v-if="mandates.length === 0"
          :title="t('registers.governance.empty')"
          size="page"
        >
          <template #icon>
            <CIconShield />
          </template>
          <template
            v-if="canEdit"
            #actions
          >
            <CBtn
              variant="primary"
              size="sm"
              :icon="CIconPlus"
              :title="t('registers.governance.new')"
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
            :rows="mandates"
            row-key="id"
          >
            <template #cell-holder_name="{ row }">
              <strong class="data-md">{{ row.holder_name }}</strong>
            </template>
            <template #cell-role="{ row }">
              <CBadge :variant="roleVariant(row.role)">
                {{ t(`registers.governance.roles.${row.role}`) }}
              </CBadge>
            </template>
            <template #cell-start_date="{ row }">
              {{ formatDate(row.start_date) }}
            </template>
            <template #cell-period="{ row }">
              <CBadge
                v-if="row.end_date === null"
                variant="valid"
              >
                {{ t('registers.governance.active') }}
              </CBadge>
              <span v-else>{{ formatDate(row.end_date) }}</span>
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

    <GovernanceMandateFormModal
      v-model:open="showModal"
      :mandate="editing"
      @saved="reload"
    />

    <GovernanceMandateDeleteModal
      v-model:open="showDeleteModal"
      :mandate="deletingMandate"
      @deleted="reload"
    />
  </div>
</template>
