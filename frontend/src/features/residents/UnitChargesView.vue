<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import CCard from '@/components/molecules/CCard.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import { unitsApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import { useAuthStore } from '@/stores/auth'
import { useFormatters } from '@/composables/useFormatters'
import type { UnitChargesResponse, UnitChargesLevy } from '@/types/domain'

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

async function load() {
  const res = await unitsApi.charges(unitId)
  data.value = res.data
}

const { loading, error, reload } = usePageLoader(load, t('common.loadError'))
onMounted(reload)

function statusClass(levy: UnitChargesLevy): string {
  return {
    issued: 'status-issued',
    paid:   'status-paid',
    overdue: 'status-overdue',
  }[levy.status] ?? ''
}

function statusLabel(levy: UnitChargesLevy): string {
  return t(`budget.levyStatuses.${levy.status}`)
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
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ data?.unit.unit_number ?? '…' }}</span>
          <CIconChevronRight
            width="14"
            height="14"
          />
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
          <p class="page-eyebrow">
            {{ t('page.eyebrows.unitCharges') }}
          </p>
          <h1 class="display-md">
            {{ t('units.charges.title') }}
          </h1>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('common.lot') }} {{ data.unit.unit_number }} · {{ data.unit.unit_type }}
          </p>
        </header>

        <!-- Summary KPIs -->
        <div class="charges-kpis">
          <div class="kpi-card">
            <span class="kpi-label">{{ t('units.charges.totalIssued') }}</span>
            <span class="kpi-value">{{ formatCurrency(data.total_issued) }}</span>
          </div>
          <div class="kpi-card">
            <span class="kpi-label">{{ t('units.charges.totalPaid') }}</span>
            <span class="kpi-value kpi-paid">{{ formatCurrency(data.total_paid) }}</span>
          </div>
          <div
            class="kpi-card"
            :class="{ 'kpi-card--warn': data.balance > 0 }"
          >
            <span class="kpi-label">{{ t('units.charges.balance') }}</span>
            <span
              class="kpi-value"
              :class="data.balance > 0 ? 'kpi-overdue' : 'kpi-paid'"
            >
              {{ formatCurrency(data.balance) }}
            </span>
          </div>
        </div>

        <!-- Legal note -->
        <p class="legal-note">
          {{ t('units.charges.legalNote') }}
        </p>

        <!-- Levy table -->
        <CCard>
          <h2 class="section-title">
            {{ t('units.charges.history') }}
          </h2>

          <div
            v-if="data.levies.length === 0"
            class="empty-state"
          >
            {{ t('units.charges.empty') }}
          </div>

          <table
            v-else
            class="data-table"
          >
            <thead>
              <tr>
                <th>{{ t('budget.fiscalYear', { year: '' }).trim() }}</th>
                <th>{{ t('common.date') }}</th>
                <th>{{ t('settings.keys.title') }}</th>
                <th>{{ t('units.shares') }}</th>
                <th>{{ t('common.amount') }}</th>
                <th>{{ t('common.status') }}</th>
                <th>{{ t('common.paidAt') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="levy in data.levies"
                :key="levy.id"
              >
                <td>{{ levy.fiscal_year }}</td>
                <td>{{ formatDate(levy.due_date) }}</td>
                <td>{{ levy.key_name ?? '—' }}</td>
                <td class="shares-cell">
                  <template v-if="levy.shares_snapshot !== null">
                    {{ levy.shares_snapshot }} / {{ levy.shares_total_snapshot }}
                  </template>
                  <template v-else>
                    —
                  </template>
                </td>
                <td class="amount-cell">
                  {{ formatCurrency(levy.amount) }}
                </td>
                <td>
                  <span
                    class="status-pill"
                    :class="statusClass(levy)"
                  >
                    {{ statusLabel(levy) }}
                  </span>
                </td>
                <td>{{ levy.paid_at ? formatDate(levy.paid_at) : '—' }}</td>
              </tr>
            </tbody>
          </table>
        </CCard>
      </template>
    </div>
  </div>
</template>

<style scoped>

.charges-kpis {
  display: flex;
  gap: var(--space-4);
  flex-wrap: wrap;
}
.kpi-card {
  flex: 1;
  min-width: 160px;
  padding: var(--space-4) var(--space-5);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.kpi-card--warn { background: color-mix(in srgb, var(--color-warning, #b45309) 8%, transparent); }
.kpi-label { font-size: var(--text-xs); color: var(--color-on-surface-variant); font-weight: var(--fw-medium); text-transform: uppercase; letter-spacing: 0.04em; }
.kpi-value { font-size: 22px; font-weight: var(--fw-bold); font-family: var(--font-display); }
.kpi-paid    { color: var(--color-success, #16a34a); }
.kpi-overdue { color: var(--color-error); }

.legal-note { font-size: var(--text-xs); color: var(--color-on-surface-variant); padding: var(--space-2) var(--space-3); border-left: 3px solid var(--color-outline-variant); margin: 0; }

.data-table { width: 100%; border-collapse: collapse; font-size: var(--text-sm); }
.data-table th, .data-table td { padding: var(--space-2) 10px; text-align: left; border-bottom: 1px solid var(--color-outline-variant); vertical-align: middle; }
.data-table th { color: var(--color-on-surface-variant); font-weight: var(--fw-medium); font-size: var(--text-xs); text-transform: uppercase; letter-spacing: 0.04em; }
.data-table tbody tr:last-child td { border-bottom: none; }

.shares-cell { font-variant-numeric: tabular-nums; color: var(--color-on-surface-variant); }
.amount-cell { font-variant-numeric: tabular-nums; font-weight: var(--fw-semibold); }

.status-pill {
  display: inline-block;
  padding: 2px var(--space-2);
  border-radius: 999px;
  font-size: var(--text-2xs);
  font-weight: var(--fw-semibold);
  white-space: nowrap;
}
.status-issued  { background: color-mix(in srgb, var(--color-primary) 12%, transparent); color: var(--color-primary); }
.status-paid    { background: color-mix(in srgb, var(--color-success, #16a34a) 12%, transparent); color: var(--color-success, #16a34a); }
.status-overdue { background: color-mix(in srgb, var(--color-error) 12%, transparent); color: var(--color-error); }

.empty-state { text-align: center; color: var(--color-on-surface-variant); padding: var(--space-6) 0; }
</style>
