<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { mockClaims, mockBuildingStats } from '@/mock/data'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()

const filter = ref<'all' | 'open' | 'in_progress' | 'closed'>('all')

const claims = computed(() =>
  filter.value === 'all' ? mockClaims : mockClaims.filter(c => c.status === filter.value)
)

function statusVariant(s: string): 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'open') return 'pending'
  if (s === 'in_progress') return 'open'
  if (s === 'closed') return 'closed'
  return 'default'
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
          <span>{{ t('claims.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('claims.create') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container">
      <CMockBanner />

      <div class="view-header">
        <div>
          <h1 class="display-md">
            {{ t('claims.title') }}
          </h1>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('claims.activeCount', mockClaims.filter(c => c.status !== 'closed').length) }}
          </p>
        </div>
      </div>

      <div class="tabs">
        <button
          v-for="opt in [
            { key: 'all', label: t('common.all') },
            { key: 'open', label: t('claims.filters.open') },
            { key: 'in_progress', label: t('claims.filters.inProgress') },
            { key: 'closed', label: t('claims.filters.closed') },
          ]"
          :key="opt.key"
          :class="['tab', { 'tab--active': filter === opt.key }]"
          @click="filter = opt.key as typeof filter"
        >
          {{ opt.label }}
        </button>
      </div>

      <div class="claims-list">
        <CCard
          v-for="claim in claims"
          :key="claim.id"
          :shadow="claim.status !== 'closed'"
          :class="['claim-card', { 'card--terminated': claim.status === 'closed' }]"
        >
          <div class="claim-header">
            <div class="claim-icon">
              🛡️
            </div>
            <div class="claim-meta-top">
              <CBadge
                :variant="statusVariant(claim.status)"
                dot
              >
                {{ t(`claims.statuses.${claim.status}`) }}
              </CBadge>
              <span class="claim-date">{{ t('claims.declaredOn', { date: formatDate(claim.declared_at) }) }}</span>
            </div>
          </div>
          <h3 class="title-md">
            {{ claim.title }}
          </h3>
          <p
            v-if="claim.description"
            class="body-sm claim-desc"
          >
            {{ claim.description }}
          </p>
          <div class="claim-footer">
            <span class="dim">{{ t('claims.openedAt', { date: formatRelative(claim.created_at) }) }}</span>
          </div>
        </CCard>
      </div>
    </div>
  </div>
</template>

<style scoped>
.claims-list { display: flex; flex-direction: column; gap: var(--space-3); }

.claim-card { display: flex; flex-direction: column; gap: var(--space-2); }
.claim-header { display: flex; align-items: center; gap: var(--space-3); }
.claim-icon { font-size: 24px; }
.claim-meta-top { display: flex; align-items: center; gap: 10px; }
.claim-date { font-size: var(--text-xs); color: var(--color-on-surface-variant); }
.claim-desc { color: var(--color-on-surface-variant); }
.claim-footer { display: flex; gap: var(--space-3); font-size: var(--text-xs); }
.dim { color: var(--color-on-surface-variant); }
</style>

