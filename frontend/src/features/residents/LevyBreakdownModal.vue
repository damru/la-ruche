<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CDataTable from '@/components/molecules/CDataTable.vue'
import { useFormatters } from '@/composables/useFormatters'
import type { UnitChargesLevy, UnitChargesLevyBreakdownItem } from '@/types/domain'
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

const props = defineProps<{
  open: boolean
  levy: ConsolidatedUnitChargeRow | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const { formatCurrency } = useFormatters()

const breakdownColumns = computed<Column<Record<string, unknown>>[]>(() => [
  { key: 'line_label', label: t('budget.lineLabel') },
  { key: 'key_name', label: t('settings.keys.title') },
  { key: 'shares', label: t('units.shares'), align: 'right', numeric: true },
  { key: 'amount', label: t('common.amount'), align: 'right', numeric: true },
])

const breakdownRows = computed<Record<string, unknown>[]>(
  () => (props.levy?.breakdown ?? []).map(d => ({
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
  (props.levy?.breakdown ?? []).reduce((sum, d) => sum + Number(d.amount), 0),
)
</script>

<template>
  <CModal
    :open="open"
    :title="levy
      ? t('units.charges.detailTitle', { label: levy.call_label })
      : t('units.charges.detailAction')"
    size="lg"
    @update:open="emit('update:open', $event)"
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
        :title="t('common.cancel')"
        @click="emit('update:open', false)"
      />
    </template>
  </CModal>
</template>
