<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import CCard from '@/components/molecules/CCard.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import LevyBreakdownModal from '@/features/residents/LevyBreakdownModal.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CSeparator from '@/components/atoms/CSeparator.vue'
import CMetaLine from '@/components/molecules/CMetaLine.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CKpiCard from '@/components/molecules/CKpiCard.vue'
import type { Column } from '@/components/molecules/CDataTable.vue'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import CIconWarn from '@/components/atoms/CIconWarn.svg'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import PreEtatDateModal from '@/features/registers/PreEtatDateModal.vue'
import { unitsApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import { usePermissions } from '@/composables/usePermissions'
import { useFormatters } from '@/composables/useFormatters'
import type { UnitChargesResponse, UnitChargesLevy, UnitChargesLevyBreakdownItem, UnitOwnerRecord } from '@/types/domain'
import { LEVY_STATUS_ISSUED, LEVY_STATUS_OVERDUE, LEVY_STATUS_PAID } from '@/utils/constants'

interface ConsolidatedUnitChargeRow {
  id: string
  fiscal_year: number
  call_label: string
  due_date: string
  issued_at: string
  paid_at: string | null
  amount: number
  status: UnitChargesLevy['status']
  breakdown: UnitChargesLevyBreakdownItem[]
}

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const { canViewUsers } = usePermissions()
const { formatCurrency, formatDate } = useFormatters()

// Co-owners go back to their personal charges list; managers go to residents
const backRoute = computed(() =>
  canViewUsers.value ? '/residents' : '/my-charges'
)
const backLabel = computed(() =>
  canViewUsers.value ? t('nav.residents') : t('nav.myCharges')
)

const unitId = route.params.id as string
const data = ref<UnitChargesResponse | null>(null)
const selectedLevy = ref<ConsolidatedUnitChargeRow | null>(null)

interface OwnerFilterOption {
  value: string
  label: string
  start_date: string
  end_date: string | null
}

const ownerFilterId = ref<string | null>(null)
const ownerFilterOptions = ref<OwnerFilterOption[]>([])
const ownerFilterIdModel = computed<string>({
  get: () => ownerFilterId.value ?? '',
  set: (v: string) => { ownerFilterId.value = v || null },
})

async function load() {
  const res = await unitsApi.charges(unitId)
  data.value = res.data

  if (canViewUsers.value) {
    try {
      const ownersRes = await unitsApi.owners(unitId)
      const seen = new Set<string>()
      const options: OwnerFilterOption[] = []
      for (const rec of ownersRes.data as UnitOwnerRecord[]) {
        if (!rec.user_id || seen.has(rec.user_id)) continue
        seen.add(rec.user_id)
        const period = rec.end_date
          ? `${rec.start_date} → ${rec.end_date}`
          : `${rec.start_date} →`
        options.push({ value: rec.user_id, label: `${rec.display_name} (${period})`, start_date: rec.start_date, end_date: rec.end_date })
      }
      ownerFilterOptions.value = options
    } catch {
      ownerFilterOptions.value = []
    }
  }
}

const { loading, error, reload } = usePageLoader(load)
onMounted(reload)

// Backend returns pre-consolidated data — one row per levy_call for this unit.
const consolidatedLevies = computed<ConsolidatedUnitChargeRow[]>(() => {
  if (!data.value) return []
  return data.value.levies.map(levy => ({
    id: levy.id,
    fiscal_year: levy.fiscal_year,
    call_label: levy.call_label,
    due_date: levy.due_date,
    issued_at: levy.issued_at,
    paid_at: levy.paid_at,
    amount: levy.total_amount,
    status: levy.status,
    breakdown: levy.breakdown,
  }))
})

const filteredLevies = computed<ConsolidatedUnitChargeRow[]>(() => {
  if (!ownerFilterId.value) return consolidatedLevies.value
  const opt = ownerFilterOptions.value.find(o => o.value === ownerFilterId.value)
  if (!opt) return consolidatedLevies.value
  return consolidatedLevies.value.filter(l =>
    l.issued_at >= opt.start_date && (opt.end_date === null || l.issued_at <= opt.end_date),
  )
})

const levyTableColumns = computed<Column<Record<string, unknown>>[]>(() => [
  { key: 'fiscal_year', label: t('budget.fiscalYear', { year: '' }).trim(), align: 'right', numeric: true },
  { key: 'issued_at', label: t('common.issuedAt'), align: 'right', numeric: true },
  { key: 'due_date', label: t('common.dueDate'), align: 'right', numeric: true },
  { key: 'call_label', label: t('common.label') },
  { key: 'amount', label: t('common.amount'), align: 'right', numeric: true },
  { key: 'status', label: t('status') },
  { key: 'paid_at', label: t('common.paidAt'), align: 'right', numeric: true },
  { key: 'actions', label: '', align: 'right', width: '1%' },
])

const levyTableRows = computed<Record<string, unknown>[]>(
  () => filteredLevies.value as unknown as Record<string, unknown>[],
)

function statusVariant(status: UnitChargesLevy['status']): 'pending' | 'valid' | 'err' | 'default' {
  if (status === LEVY_STATUS_ISSUED) return 'pending'
  if (status === LEVY_STATUS_PAID) return 'valid'
  if (status === LEVY_STATUS_OVERDUE) return 'err'
  return 'default'
}

const showPreEtatDateModal = ref(false)

</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          <span
            class="cursor-pointer"
            @click="router.push(backRoute)"
          >{{ backLabel }}</span>
          <CIconChevronRight />
          <span>{{ data?.unit.unit_number ?? '…' }}</span>
          <CIconChevronRight />
          <span>{{ t('units.charges.title') }}</span>
        </div>
      </template>
    </CTopbar>

    <div class="view-container">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else-if="data">
        <header class="view-header">
          <CEyebrow>
            {{ t('page.eyebrows.unitCharges') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('units.charges.title') }}
          </CHeading>
          <CMetaLine>
            <CMetaItem variant="sub">
              {{ t('common.lot') }} {{ data.unit.unit_number }} 
            </CMetaItem>
            <CSeparator />
            <CMetaItem variant="sub">
              {{ t(`units.types.${data.unit.unit_type}`) }}
            </CMetaItem>
          </CMetaLine>
        </header>

        <div
          v-if="canViewUsers"
          class="flex justify-end"
        >
          <CBtn
            variant="ghost"
            size="sm"
            :title="t('registers.preEtatDate.documentTitle')"
            @click="showPreEtatDateModal = true"
          />
        </div>

        <CBadge
          v-if="data.pending_transfer"
          variant="warn"
          class="flex items-center gap-1"
        >
          <CIconWarn class="size-4 shrink-0" />
          {{ t('units.charges.pendingTransfer') }}
        </CBadge>

        <!-- Summary KPIs -->
        <div class="flex flex-wrap gap-4">
          <CKpiCard
            :label="t('units.charges.totalIssued')"
            :value="formatCurrency(data.total_issued)"
          />
          <CKpiCard
            :label="t('units.charges.totalPaid')"
            :value="formatCurrency(data.total_paid)"
            tone="valid"
            :emphasize="true"
          />
          <CKpiCard
            :label="t('units.charges.balance')"
            :value="formatCurrency(data.balance)"
            :tone="data.balance > 0 ? 'err' : 'valid'"
            :emphasize="true"
          />
        </div>

        <CCard>
          <div class="flex items-center justify-between mb-4">
            <CHeading type="card">
              {{ t('units.charges.history') }}
            </CHeading>
            <CFormSelect
              v-if="canViewUsers && ownerFilterOptions.length > 1"
              v-model="ownerFilterIdModel"
              :placeholder="t('budget.filterByOwner')"
              :options="[{ value: '', label: t('common.allMasc') }, ...ownerFilterOptions]"
              class="min-w-48"
            />
          </div>

          <CDataTable
            :columns="levyTableColumns"
            :rows="levyTableRows"
            :empty-text="t('units.charges.empty')"
            class="-mx-5"
          >
            <template #cell-issued_at="{ row }">
              {{ formatDate(row.issued_at as string) }}
            </template>
            <template #cell-due_date="{ row }">
              {{ formatDate(row.due_date as string) }}
            </template>
            <template #cell-amount="{ row }">
              <span class="cell-numeric">
                {{ formatCurrency(Number(row.amount)) }}
              </span>
            </template>
            <template #cell-status="{ row }">
              <CBadge
                :variant="statusVariant(row.status as UnitChargesLevy['status'])"
                :pulse="row.status === LEVY_STATUS_OVERDUE"
                dot
                :ring="row.status === LEVY_STATUS_ISSUED || row.status === LEVY_STATUS_OVERDUE"
              >
                {{ t(`budget.levyStatuses.${row.status}`) }}
              </CBadge>
            </template>
            <template #cell-paid_at="{ row }">
              {{ row.paid_at ? formatDate(row.paid_at as string) : '—' }}
            </template>
            <template #cell-actions="{ row }">
              <CBtn
                variant="ghost"
                size="sm"
                :title="t('common.details')"
                @click="selectedLevy = consolidatedLevies.find(l => l.id === row.id) ?? null"
              />
            </template>
          </CDataTable>
        </CCard>

        <CLegalCard>
          <CFlexGroup>
            <CLegalNotice
              :links="[{ label: t('units.charges.legalCard.art10linkLabel'), href: t('units.charges.legalCard.art10linkHref') }]"
            >
              <CLegalDescription
                :title="t('units.charges.legalCard.art10title')"
                :summary="t('units.charges.legalCard.art10summary')"
              />
            </CLegalNotice>
            <CLegalNotice
              :links="[{ label: t('units.charges.legalCard.art18linkLabel'), href: t('units.charges.legalCard.art18linkHref') }]"
            >
              <CLegalDescription
                :title="t('units.charges.legalCard.art18title')"
                :summary="t('units.charges.legalCard.art18summary')"
              />
            </CLegalNotice>
          </CFlexGroup>
        </CLegalCard>

        <LevyBreakdownModal
          :open="!!selectedLevy"
          :levy="selectedLevy"
          @update:open="open => { if (!open) selectedLevy = null }"
        />
        <PreEtatDateModal
          v-model:open="showPreEtatDateModal"
          :unit-id="unitId"
          :unit-number="data.unit.unit_number"
        />
      </template>
    </div>
  </div>
</template>
