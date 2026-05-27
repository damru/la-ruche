<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import CCard from '@/components/molecules/CCard.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CModal from '@/components/molecules/CModal.vue'
import CDataTable from '@/components/molecules/CDataTable.vue'
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
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import { unitsApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import { useAuthStore } from '@/stores/auth'
import { useFormatters } from '@/composables/useFormatters'
import type { UnitChargesResponse, UnitChargesLevy, UnitChargesLevyBreakdownItem } from '@/types/domain'
import type { Column } from '@/components/molecules/CDataTable.vue'

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
const auth = useAuthStore()
const { formatCurrency, formatDate } = useFormatters()

// Co-owners go back to their personal charges list; managers go to residents
const backRoute = computed(() =>
  auth.isManager || auth.isBoardMember ? '/residents' : '/my-charges'
)
const backLabel = computed(() =>
  auth.isManager || auth.isBoardMember ? t('nav.residents') : t('nav.myCharges')
)

const unitId = route.params.id as string
const data = ref<UnitChargesResponse | null>(null)
const selectedLevy = ref<ConsolidatedUnitChargeRow | null>(null)

async function load() {
  const res = await unitsApi.charges(unitId)
  data.value = res.data
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

const levyTableColumns = computed<Column<Record<string, unknown>>[]>(() => [
  { key: 'fiscal_year', label: t('budget.fiscalYear', { year: '' }).trim(), align: 'right', numeric: true },
  { key: 'issued_at', label: t('common.issuedAt'), align: 'right', numeric: true },
  { key: 'due_date', label: t('common.dueDate'), align: 'right', numeric: true },
  { key: 'call_label', label: t('budget.levyCallLabel') },
  { key: 'amount', label: t('common.amount'), align: 'right', numeric: true },
  { key: 'status', label: t('common.status') },
  { key: 'paid_at', label: t('common.paidAt'), align: 'right', numeric: true },
  { key: 'actions', label: '', align: 'right', width: '1%' },
])

const levyTableRows = computed<Record<string, unknown>[]>(
  () => consolidatedLevies.value as unknown as Record<string, unknown>[],
)

const breakdownColumns = computed<Column<Record<string, unknown>>[]>(() => [
  { key: 'line_label', label: t('budget.lineLabel') },
  { key: 'key_name', label: t('settings.keys.title') },
  { key: 'shares', label: t('units.shares'), align: 'right', numeric: true },
  { key: 'amount', label: t('common.amount'), align: 'right', numeric: true },
])

const breakdownRows = computed<Record<string, unknown>[]>(
  () => (selectedLevy.value?.breakdown ?? []).map(d => ({
    ...d,
    line_label: d.is_works_fund
      ? t('budget.worksFund')
      : (d.budget_line_label ?? '—'),
    shares: d.shares_snapshot !== null
      ? `${d.shares_snapshot} / ${d.shares_total_snapshot}`
      : null,
  })) as unknown as Record<string, unknown>[],
)

const breakdownTotal = computed(() =>
  (selectedLevy.value?.breakdown ?? []).reduce((sum, d) => sum + Number(d.amount), 0),
)

function statusVariant(status: UnitChargesLevy['status']): 'pending' | 'valid' | 'err' | 'default' {
  if (status === 'issued') return 'pending'
  if (status === 'paid') return 'valid'
  if (status === 'overdue') return 'err'
  return 'default'
}

</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          <span
            style="cursor:pointer"
            @click="router.push(backRoute)"
          >{{ backLabel }}</span>
          <CIconChevronRight />
          <span>{{ data?.unit.unit_number ?? '…' }}</span>
          <CIconChevronRight />
          <span>{{ t('units.charges.title') }}</span>
        </div>
      </template>
    </CTopbar>

    <div class="flex flex-col p-10 max-w-[1200px] gap-6 max-sm:p-5">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else-if="data">
        <header class="flex flex-col items-start gap-2">
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
          <CHeading type="card" class="mb-4">
            {{ t('units.charges.history') }}
          </CHeading>

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
                :pulse="row.status === 'overdue'"
                dot
                :ring="row.status === 'issued' || row.status === 'overdue'"
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
                @click="selectedLevy = consolidatedLevies.find(l => l.id === row.id) ?? null"
              >
                {{ t('units.charges.detailAction') }}
              </CBtn>
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

        <CModal
          :open="!!selectedLevy"
          :title="selectedLevy
            ? t('units.charges.detailTitle', { label: selectedLevy.call_label })
            : t('units.charges.detailAction')"
          size="lg"
          @update:open="open => { if (!open) selectedLevy = null }"
        >
          <CDataTable
            :columns="breakdownColumns"
            :rows="breakdownRows"
            class="-mx-5 bg-transparent! border-b border-outline-variant"
          >
            <template #cell-key_name="{ row }">
              {{ row.key_name ?? '—' }}
            </template>
            <template #cell-shares="{ row }">
              <span class="text-on-surface-variant">{{ row.shares ?? '—' }}</span>
            </template>
            <template #cell-amount="{ row }">
              <span class="cell-numeric">
                {{ formatCurrency(Number(row.amount)) }}
              </span>
            </template>
          </CDataTable>
          <div class="flex items-center justify-between font-semibold text-sm mt-1">
            <strong>{{ t('common.total') }}</strong>
            <p class="justify-end font-mono">
              {{ formatCurrency(breakdownTotal) }}
            </p>
          </div>
          <template #footer>
            <CBtn
              variant="ghost"
              @click="selectedLevy = null"
            >
              {{ t('common.cancel') }}
            </CBtn>
          </template>
        </CModal>
      </template>
    </div>
  </div>
</template>

<style scoped>
</style>
