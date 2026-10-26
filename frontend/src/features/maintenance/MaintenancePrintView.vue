<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CDocumentHeader from '@/components/molecules/CDocumentHeader.vue'
import CSectionHeader from '@/components/molecules/CSectionHeader.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CIconArrowLeft from '@/components/atoms/CIconArrowLeft.svg'
import { maintenanceApi } from '@/api'
import type { LogbookContractSummary, LogbookSummary, MaintenanceCategory, MaintenanceEntry } from '@/types/domain'

const { t } = useI18n()
const router = useRouter()
const { formatDate, formatCurrency } = useFormatters()

const logbookSummary = ref<LogbookSummary | null>(null)
const allEntries = ref<MaintenanceEntry[]>([])

async function loadData() {
  const [summaryRes, entriesRes] = await Promise.all([
    maintenanceApi.summary(),
    maintenanceApi.list(),
  ])
  logbookSummary.value = summaryRes.data
  allEntries.value = entriesRes.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

const building = computed(() => logbookSummary.value?.building)

const buildingAddress = computed(() => {
  const b = building.value
  if (!b) return ''
  return [b.address_line1, b.address_line2, `${b.postal_code} ${b.city}`].filter(Boolean).join(', ')
})

const managerLabel = computed(() => {
  const manager = logbookSummary.value?.current_manager
  if (!manager) return t('maintenance.regulatory.noManager')
  return `${manager.first_name} ${manager.last_name}`.trim()
})

const headerMeta = computed(() => [
  { label: t('maintenance.regulatory.building'), value: buildingAddress.value },
  { label: t('maintenance.regulatory.manager'), value: managerLabel.value },
])

const sortedEntries = computed(() =>
  [...allEntries.value].sort((a, b) => a.intervention_date.localeCompare(b.intervention_date)),
)

const insuranceContracts = computed(() => logbookSummary.value?.insurance_contracts ?? [])
const maintenanceContracts = computed(() => logbookSummary.value?.maintenance_contracts ?? [])

function categoryVariant(category: MaintenanceCategory): 'primary' | 'accent' | 'warn' | 'default' {
  if (category === 'travaux') return 'primary'
  if (category === 'contrat') return 'accent'
  if (category === 'sinistre') return 'warn'
  return 'default'
}

function contractPeriod(contract: LogbookContractSummary): string {
  const start = formatDate(contract.start_date)
  return contract.end_date
    ? `${start} – ${formatDate(contract.end_date)}`
    : `${start} – …`
}

function print() {
  window.print()
}

function goBack() {
  router.push({ name: 'maintenance' })
}
</script>

<template>
  <div class="print-page">
    <div class="print-toolbar print:hidden">
      <CBtn
        variant="ghost"
        size="sm"
        :icon="CIconArrowLeft"
        :title="t('maintenance.print.backToLogbook')"
        @click="goBack"
      />
      <CBtn
        variant="primary"
        size="sm"
        :title="t('maintenance.print.printButton')"
        @click="print"
      />
    </div>

    <CPageError
      v-if="error"
      :message="error"
      @retry="reload"
    />
    <CPageLoader v-else-if="loading" />
    <article
      v-else
      class="print-document"
    >
      <CDocumentHeader
        :title="t('maintenance.print.title')"
        :subtitle="building?.name"
        :meta="headerMeta"
        :issued-on="new Date()"
      />

      <section class="print-section">
        <CSectionHeader>
          <span class="title-sm">{{ t('maintenance.print.insuranceSection') }}</span>
          <template #actions>
            <CBadge variant="default">
              {{ t('contracts.count', insuranceContracts.length) }}
            </CBadge>
          </template>
        </CSectionHeader>
        <CCard
          v-if="insuranceContracts.length > 0"
          :level="3"
        >
          <ul class="contract-table">
            <li
              v-for="contract in insuranceContracts"
              :key="contract.id"
              class="contract-row"
            >
              <div class="contract-row__main">
                <span class="contract-row__subject">{{ contract.subject }}</span>
                <span class="contract-row__vendor">{{ contract.vendor_name }}</span>
              </div>
              <div class="contract-row__aside">
                <span>{{ contractPeriod(contract) }}</span>
                <span v-if="contract.amount != null">{{ formatCurrency(contract.amount) }}</span>
              </div>
            </li>
          </ul>
        </CCard>
        <p
          v-else
          class="print-empty"
        >
          {{ t('maintenance.print.noContracts') }}
        </p>
      </section>

      <section class="print-section">
        <CSectionHeader>
          <span class="title-sm">{{ t('maintenance.print.maintenanceSection') }}</span>
          <template #actions>
            <CBadge variant="default">
              {{ t('contracts.count', maintenanceContracts.length) }}
            </CBadge>
          </template>
        </CSectionHeader>
        <CCard
          v-if="maintenanceContracts.length > 0"
          :level="3"
        >
          <ul class="contract-table">
            <li
              v-for="contract in maintenanceContracts"
              :key="contract.id"
              class="contract-row"
            >
              <div class="contract-row__main">
                <span class="contract-row__subject">{{ contract.subject }}</span>
                <span class="contract-row__vendor">
                  {{ contract.vendor_name }} · {{ t(`contracts.types.${contract.contract_type}`) }}
                </span>
              </div>
              <div class="contract-row__aside">
                <span>{{ contractPeriod(contract) }}</span>
                <span v-if="contract.amount != null">{{ formatCurrency(contract.amount) }}</span>
              </div>
            </li>
          </ul>
        </CCard>
        <p
          v-else
          class="print-empty"
        >
          {{ t('maintenance.print.noContracts') }}
        </p>
      </section>

      <section class="print-section">
        <CSectionHeader>
          <span class="title-sm">{{ t('maintenance.print.chronologySection') }}</span>
          <template #actions>
            <CBadge variant="default">
              {{ t('maintenance.header.entriesCount', sortedEntries.length) }}
            </CBadge>
          </template>
        </CSectionHeader>
        <table
          v-if="sortedEntries.length > 0"
          class="entries-table"
        >
          <thead>
            <tr>
              <th scope="col">
                {{ t('maintenance.print.table.date') }}
              </th>
              <th scope="col">
                {{ t('maintenance.print.table.title') }}
              </th>
              <th scope="col">
                {{ t('maintenance.print.table.vendor') }}
              </th>
              <th scope="col">
                {{ t('maintenance.print.table.category') }}
              </th>
              <th scope="col">
                {{ t('maintenance.print.table.source') }}
              </th>
              <th
                scope="col"
                class="entries-table__cost"
              >
                {{ t('maintenance.print.table.cost') }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="entry in sortedEntries"
              :key="entry.id"
              class="entries-table__row"
            >
              <td>{{ formatDate(entry.intervention_date) }}</td>
              <td>
                <span class="entries-table__entry-title">{{ entry.title }}</span>
                <span
                  v-if="entry.description"
                  class="entries-table__entry-desc"
                >{{ entry.description }}</span>
              </td>
              <td>{{ entry.vendor_name ?? '—' }}</td>
              <td>
                <CBadge :variant="categoryVariant(entry.category)">
                  {{ t(`maintenance.categories.${entry.category}`) }}
                </CBadge>
              </td>
              <td>{{ t(`maintenance.sources.${entry.source}`) }}</td>
              <td class="entries-table__cost">
                {{ entry.cost != null ? formatCurrency(entry.cost) : '—' }}
              </td>
            </tr>
          </tbody>
        </table>
        <p
          v-else
          class="print-empty"
        >
          {{ t('maintenance.print.noEntries') }}
        </p>
      </section>

      <footer class="print-footer">
        {{ t('maintenance.print.editedOn', { date: formatDate(new Date().toISOString(), 'long') }) }}
      </footer>
    </article>
  </div>
</template>

<style scoped>
.print-page {
  min-height: 100vh;
  background: var(--color-surface);
}

.print-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-4);
  border-bottom: 1px solid var(--color-outline-variant);
}

.print-document {
  max-width: 210mm;
  margin: 0 auto;
  padding: var(--space-8);
  background: var(--color-surface);
  color: var(--color-on-surface);
}

.print-section {
  margin-bottom: var(--space-8);
  break-inside: avoid;
}

.print-empty {
  font-size: var(--text-sm);
  color: var(--color-on-surface-variant);
  margin: 0;
}

.contract-table {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
}

.contract-row {
  display: flex;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-2) 0;
  break-inside: avoid;
}

.contract-row + .contract-row {
  border-top: 1px solid var(--color-outline-variant);
}

.contract-row__main {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.contract-row__subject {
  font-size: var(--text-sm);
  font-weight: var(--fw-semibold);
}

.contract-row__vendor {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
}

.contract-row__aside {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
  flex-shrink: 0;
}

.entries-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--text-xs);
}

.entries-table th {
  text-align: left;
  font-weight: var(--fw-semibold);
  color: var(--color-on-surface-variant);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-size: var(--text-2xs);
  padding: var(--space-2);
  border-bottom: 1px solid var(--color-outline-variant);
}

.entries-table__row {
  break-inside: avoid;
}

.entries-table__row td {
  padding: var(--space-2);
  border-bottom: 1px solid var(--color-outline-variant);
  vertical-align: top;
}

.entries-table__entry-title {
  display: block;
  font-weight: var(--fw-medium, 500);
}

.entries-table__entry-desc {
  display: block;
  color: var(--color-on-surface-variant);
  margin-top: 2px;
}

.entries-table__cost {
  text-align: right;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.entries-table th.entries-table__cost {
  text-align: right;
}

.print-footer {
  margin-top: var(--space-8);
  padding-top: var(--space-4);
  border-top: 1px solid var(--color-outline-variant);
  text-align: center;
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
}

@media print {
  @page {
    size: A4;
    margin: 16mm;
  }

  .print-page {
    background: #fff;
  }

  .print-document {
    max-width: none;
    padding: 0;
    background: #fff;
    color: #000;
  }

  .entries-table__row {
    break-inside: avoid;
  }

  thead {
    display: table-header-group;
  }

  .print-footer {
    color: #333;
    border-top-color: #000;
  }
}
</style>
