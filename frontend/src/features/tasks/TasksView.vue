<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.vue'
import CIconPlus from '@/components/atoms/CIconPlus.vue'
import CIconDotsHorizontal from '@/components/atoms/CIconDotsHorizontal.vue'
import { mockTasks, mockBuildingStats } from '@/mock/data'
import type { TaskStatus } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()

const statusFilter = ref<'all' | TaskStatus>('all')

const tasks = computed(() =>
  statusFilter.value === 'all'
    ? mockTasks
    : mockTasks.filter(t => t.status === statusFilter.value)
)

const counts = computed(() => ({
  open: mockTasks.filter(t => t.status === 'open').length,
  in_progress: mockTasks.filter(t => t.status === 'in_progress').length,
  awaiting_vendor: mockTasks.filter(t => t.status === 'awaiting_vendor').length,
  closed: mockTasks.filter(t => t.status === 'closed').length,
}))

function statusVariant(s: string): 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'open') return 'pending'
  if (s === 'in_progress') return 'open'
  if (s === 'awaiting_vendor') return 'default'
  if (s === 'closed') return 'closed'
  return 'default'
}

function priorityVariant(p: string): 'open' | 'pending' | 'closed' | 'default' {
  if (p === 'urgent') return 'closed'
  if (p === 'high') return 'open'
  if (p === 'normal') return 'pending'
  return 'default'
}

function priorityColor(p: string): string {
  if (p === 'urgent') return 'var(--color-error)'
  if (p === 'high') return 'var(--color-warning)'
  if (p === 'normal') return 'var(--color-primary)'
  return 'var(--color-outline-variant)'
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ mockBuildingStats.name }}
          <CIconChevronRight width="14" height="14"/>
          <span>{{ t('tasks.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn variant="primary" size="sm">
          <CIconPlus style="width:16px;height:16px"/>
          Nouvelle tâche
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container">
    <CMockBanner />

    <div class="view-header">
      <div>
        <h1 class="display-md">{{ t('tasks.title') }}</h1>
        <p class="body-md" style="color:var(--color-on-surface-variant)">
          {{ mockTasks.filter(t => t.status !== 'closed' && t.status !== 'cancelled').length }} tâche(s) actives
        </p>
      </div>
    </div>

    <!-- Filtres -->
    <div class="tabs">
      <button
        v-for="item in [{ key: 'all', label: 'Toutes', count: mockTasks.length }, { key: 'open', label: 'Ouvertes', count: counts.open }, { key: 'in_progress', label: 'En cours', count: counts.in_progress }, { key: 'awaiting_vendor', label: 'Prestataire', count: counts.awaiting_vendor }, { key: 'closed', label: 'Clôturées', count: counts.closed }]"
        :key="item.key"
        :class="['tab', { 'tab--active': statusFilter === item.key }]"
        @click="statusFilter = item.key as typeof statusFilter"
      >
        {{ item.label }}
        <span class="tab-count">{{ item.count }}</span>
      </button>
    </div>

    <!-- Liste -->
    <div class="task-list">
      <CCard v-for="task in tasks" :key="task.id" padding="none" class="task-card">
        <div class="task-row">
          <div class="prio-bar" :style="{ background: priorityColor(task.priority) }" />
          <div class="task-body">
            <div class="task-head">
              <span class="title-sm">{{ task.title }}</span>
              <div class="task-badges">
                <CBadge :variant="priorityVariant(task.priority)">
                  {{ t(`tasks.priorities.${task.priority}`) }}
                </CBadge>
                <CBadge :variant="statusVariant(task.status)">
                  {{ t(`tasks.statuses.${task.status}`) }}
                </CBadge>
              </div>
            </div>
            <p v-if="task.description" class="task-desc body-sm">{{ task.description }}</p>
            <div class="task-meta">
              <span>Créée {{ formatRelative(task.created_at) }}</span>
              <span v-if="task.closed_at">• Clôturée le {{ formatDate(task.closed_at) }}</span>
            </div>
          </div>
          <button class="icon-btn">
            <CIconDotsHorizontal/>
          </button>
        </div>
      </CCard>

      <div v-if="!tasks.length" class="empty-state">
        Aucune tâche dans cette catégorie.
      </div>
    </div>
  </div>
  </div>
</template>

<style scoped>
.view-container { padding: 40px; max-width: 900px; display: flex; flex-direction: column; gap: 24px; }
.view-header { display: flex; align-items: flex-start; justify-content: space-between; }

.task-list { display: flex; flex-direction: column; gap: 10px; }

.task-card { overflow: hidden; }
.task-row { display: flex; align-items: stretch; gap: 0; }

.prio-bar { width: 4px; flex-shrink: 0; }

.task-body { flex: 1; padding: 14px 16px; min-width: 0; }
.task-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 6px; }
.task-badges { display: flex; gap: 6px; flex-shrink: 0; }
.task-desc { color: var(--color-on-surface-variant); margin-bottom: 8px; }
.task-meta { font-size: 11px; color: var(--color-on-surface-variant); }

.icon-btn {
  padding: 14px 12px; background: none; border: none;
  color: var(--color-on-surface-variant); cursor: pointer;
  transition: color var(--transition-fast);
}
.icon-btn:hover { color: var(--color-primary); }
.icon-btn svg { width: 18px; height: 18px; }

.empty-state { padding: 32px; text-align: center; font-size: 14px; color: var(--color-on-surface-variant); }
</style>

