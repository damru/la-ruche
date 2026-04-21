<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { mockContracts, mockReminders, mockBuildingStats } from '@/mock/data'

const { t } = useI18n()
const { formatCurrency, formatDate, daysUntil } = useFormatters()

const filter = ref<'all' | 'active' | 'expired'>('all')

const contracts = computed(() => {
  if (filter.value === 'active') return mockContracts.filter(c => c.status === 'active')
  if (filter.value === 'expired') return mockContracts.filter(c => c.status !== 'active')
  return mockContracts
})

function statusVariant(s: string): 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'active') return 'open'
  if (s === 'terminated') return 'closed'
  if (s === 'expired') return 'closed'
  return 'default'
}

function typeIcon(type: string): string {
  const icons: Record<string, string> = { maintenance: '🔧', insurance: '🛡️', service: '🧹', other: '📄' }
  return icons[type] ?? '📄'
}

function reminderUrgency(dateStr: string): 'high' | 'medium' | 'low' {
  const d = daysUntil(dateStr)
  return d <= 30 ? 'high' : d <= 90 ? 'medium' : 'low'
}

function reminderFor(contractId: string) {
  return mockReminders.filter(r => r.contract_id === contractId && r.status === 'pending')
}

function freqLabel(freq?: string): string {
  return freq ? t(`contracts.frequencies.${freq}`) : '—'
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ mockBuildingStats.name }}
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('contracts.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('contracts.create') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container">
      <CMockBanner />

      <div class="view-header">
        <div>
          <h1 class="display-md">
            {{ t('contracts.title') }}
          </h1>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('contracts.count', mockContracts.length) }}
          </p>
        </div>
      </div>

      <!-- Rappels à venir -->
      <CCard padding="none">
        <div class="section-head">
          <h2 class="section-title">
            {{ t('contracts.upcomingReminders') }}
          </h2>
        </div>
        <div class="reminder-list">
          <div
            v-for="r in mockReminders.filter(r => r.status === 'pending').sort((a,b) => new Date(a.reminder_date).getTime() - new Date(b.reminder_date).getTime())"
            :key="r.id"
            class="reminder-row"
            :class="`reminder-row--${reminderUrgency(r.reminder_date)}`"
          >
            <div class="reminder-dot"></div>
            <div class="reminder-info">
              <span class="reminder-contract">{{ mockContracts.find(c => c.id === r.contract_id)?.vendor_name }}</span>
              <span class="reminder-type">{{ t(`contracts.reminderTypes.${r.reminder_type}`) }}</span>
            </div>
            <span class="reminder-date">{{ formatDate(r.reminder_date) }}</span>
            <span
              class="reminder-days"
              :class="`urgency-${reminderUrgency(r.reminder_date)}`"
            >
              {{ t('format.daysLeft', { days: daysUntil(r.reminder_date) }) }}
            </span>
          </div>
        </div>
      </CCard>

      <!-- Filtres -->
      <div class="tabs">
        <button
          v-for="opt in [
            { key: 'all', label: t('common.all') },
            { key: 'active', label: t('contracts.filters.active') },
            { key: 'expired', label: t('contracts.filters.expired') },
          ]"
          :key="opt.key"
          :class="['tab', { 'tab--active': filter === opt.key }]"
          @click="filter = opt.key as typeof filter"
        >
          {{ opt.label }}
        </button>
      </div>

      <!-- Contracts grid -->
      <div class="contracts-grid">
        <CCard
          v-for="c in contracts"
          :key="c.id"
          class="contract-card"
        >
          <div class="contract-header">
            <span class="type-icon">{{ typeIcon(c.contract_type) }}</span>
            <CBadge
              :variant="statusVariant(c.status)"
              dot
            >
              {{ t(`contracts.statuses.${c.status}`) }}
            </CBadge>
          </div>
          <h3 class="title-md vendor-name">
            {{ c.vendor_name }}
          </h3>
          <p class="body-sm contract-subject">
            {{ c.subject }}
          </p>

          <div class="contract-details">
            <div class="detail-row">
              <span class="detail-label">{{ t('common.startDate') }}</span>
              <span>{{ formatDate(c.start_date) }}</span>
            </div>
            <div
              v-if="c.end_date"
              class="detail-row"
            >
              <span class="detail-label">{{ t('common.endDate') }}</span>
              <span>{{ formatDate(c.end_date) }}</span>
            </div>
            <div
              v-if="c.amount"
              class="detail-row"
            >
              <span class="detail-label">{{ t('common.amount') }}</span>
              <span>{{ formatCurrency(c.amount) }} / {{ freqLabel(c.payment_frequency) }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">{{ t('contracts.autoRenewal') }}</span>
              <span>{{ c.auto_renewal ? t('contracts.autoRenewalAuto', { days: c.notice_days }) : t('contracts.autoRenewalManual') }}</span>
            </div>
          </div>

          <div
            v-if="reminderFor(c.id).length"
            class="contract-reminders"
          >
            <CBadge
              v-for="r in reminderFor(c.id)"
              :key="r.id"
              variant="pending"
            >
              🔔 {{ t(`contracts.reminderTypes.${r.reminder_type}`) }} — {{ formatDate(r.reminder_date) }}
            </CBadge>
          </div>

          <div class="contract-contacts">
            <a
              v-if="c.vendor_email"
              :href="`mailto:${c.vendor_email}`"
              class="contact-link"
            >✉️ {{ c.vendor_email }}</a>
            <a
              v-if="c.vendor_phone"
              :href="`tel:${c.vendor_phone}`"
              class="contact-link"
            >📞 {{ c.vendor_phone }}</a>
          </div>
        </CCard>
      </div>
    </div>
  </div>
</template>

<style scoped>
.section-head { padding: var(--space-4) var(--space-5) 0; }

.reminder-list { padding: var(--space-3) var(--space-5) var(--space-4); display: flex; flex-direction: column; gap: var(--space-2); }
.reminder-row { display: flex; align-items: center; gap: var(--space-3); padding: var(--space-2) var(--space-3); border-radius: var(--radius-sm); }
.reminder-row--high { background: var(--color-error-container); }
.reminder-row--medium { background: var(--color-warning-container); }
.reminder-dot { width: 8px; height: 8px; border-radius: 50%; background: currentColor; flex-shrink: 0; }
.reminder-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.reminder-contract { font-size: var(--text-sm); font-weight: var(--fw-medium); }
.reminder-type { font-size: var(--text-2xs); color: var(--color-on-surface-variant); }
.reminder-date { font-size: var(--text-sm); }
.reminder-days { font-size: var(--text-xs); font-weight: var(--fw-semibold); flex-shrink: 0; }
.urgency-high { color: var(--color-error); }
.urgency-medium { color: var(--color-warning); }
.urgency-low { color: var(--color-on-surface-variant); }

.contracts-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: var(--space-4); }

.contract-card { display: flex; flex-direction: column; gap: 10px; }
.contract-header { display: flex; align-items: center; justify-content: space-between; }
.type-icon { font-size: 24px; }
.vendor-name { margin: 0; }
.contract-subject { color: var(--color-on-surface-variant); }

.contract-details { display: flex; flex-direction: column; gap: var(--space-1); }
.detail-row { display: flex; justify-content: space-between; font-size: var(--text-sm); }
.detail-label { color: var(--color-on-surface-variant); }

.contract-reminders { display: flex; flex-wrap: wrap; gap: 6px; }

.contract-contacts { display: flex; flex-direction: column; gap: var(--space-1); }
.contact-link { font-size: var(--text-xs); color: var(--color-primary); text-decoration: none; }
.contact-link:hover { text-decoration: underline; }
</style>

