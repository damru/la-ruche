<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.vue'
import CIconPlus from '@/components/atoms/CIconPlus.vue'
import CIconInfo from '@/components/atoms/CIconInfo.vue'
import { mockReports, mockUnits, mockUsers, mockBuildingStats } from '@/mock/data'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()

const filter = ref<'all' | 'open' | 'in_progress' | 'closed'>('all')

const reports = computed(() =>
  filter.value === 'all' ? mockReports : mockReports.filter(r => r.status === filter.value)
)

function statusVariant(s: string): 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'open') return 'pending'
  if (s === 'in_progress') return 'open'
  if (s === 'closed') return 'closed'
  return 'default'
}

function unitLabel(unitId?: string): string {
  if (!unitId) return '—'
  const u = mockUnits.find(u => u.id === unitId)
  return u ? t('reports.unitLabel', { number: u.unit_number }) : '—'
}

function authorName(authorId: string): string {
  const u = mockUsers.find(u => u.id === authorId)
  return u ? `${u.first_name} ${u.last_name}` : '—'
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ mockBuildingStats.name }}
          <CIconChevronRight width="14" height="14"/>
          <span>{{ t('reports.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn variant="primary" size="sm">
          <CIconPlus style="width:16px;height:16px"/>
          {{ t('reports.create') }}
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container">
    <CMockBanner />

    <div class="view-header">
      <div>
        <h1 class="display-md">{{ t('reports.title') }}</h1>
        <p class="body-md" style="color:var(--color-on-surface-variant)">
          {{ t('reports.activeCount', mockReports.filter(r => r.status !== 'closed').length) }}
        </p>
      </div>
    </div>

    <div class="tabs">
      <button
        v-for="opt in [
          { key: 'all', label: t('common.all') },
          { key: 'open', label: t('reports.filters.open') },
          { key: 'in_progress', label: t('reports.filters.inProgress') },
          { key: 'closed', label: t('reports.filters.closed') },
        ]"
        :key="opt.key"
        :class="['tab', { 'tab--active': filter === opt.key }]"
        @click="filter = opt.key as typeof filter"
      >{{ opt.label }}</button>
    </div>

    <div class="report-list">
      <CCard v-for="r in reports" :key="r.id" class="report-card">
        <div class="report-header">
          <div class="report-icon">
            <CIconInfo/>
          </div>
          <div class="report-meta">
            <CBadge :variant="statusVariant(r.status)" dot>
              {{ t(`reports.statuses.${r.status}`) }}
            </CBadge>
            <span class="report-date">{{ formatRelative(r.created_at) }}</span>
          </div>
        </div>
        <h3 class="title-sm report-title">{{ r.title }}</h3>
        <p v-if="r.description" class="body-sm report-desc">{{ r.description }}</p>
        <div class="report-footer">
          <span class="dim">👤 {{ authorName(r.author_id) }}</span>
          <span class="dim">🏠 {{ unitLabel(r.unit_id) }}</span>
          <span class="dim">📅 {{ formatDate(r.created_at) }}</span>
        </div>
      </CCard>
    </div>
  </div>
  </div>
</template>

<style scoped>
.view-container { padding: 40px; max-width: 900px; display: flex; flex-direction: column; gap: 24px; }
.view-header { display: flex; align-items: flex-start; justify-content: space-between; }

.report-list { display: flex; flex-direction: column; gap: 12px; }
.report-card { display: flex; flex-direction: column; gap: 8px; }

.report-header { display: flex; align-items: center; gap: 12px; }
.report-icon {
  width: 36px; height: 36px; border-radius: var(--radius-sm);
  background: var(--color-surface-container);
  display: flex; align-items: center; justify-content: center; flex-shrink: 0;
}
.report-icon svg { width: 18px; height: 18px; color: var(--color-on-surface-variant); }
.report-meta { display: flex; align-items: center; gap: 8px; }
.report-date { font-size: 12px; color: var(--color-on-surface-variant); }
.report-title { color: var(--color-on-surface); }
.report-desc { color: var(--color-on-surface-variant); }
.report-footer { display: flex; gap: 14px; font-size: 12px; flex-wrap: wrap; }
.dim { color: var(--color-on-surface-variant); }
</style>
