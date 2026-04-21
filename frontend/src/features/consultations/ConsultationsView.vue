<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { mockConsultations, mockBuildingStats } from '@/mock/data'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()

const filter = ref<'all' | 'open' | 'closed'>('all')

const consultations = computed(() =>
  filter.value === 'all' ? mockConsultations : mockConsultations.filter(c => c.status === filter.value)
)

// Faux résultats de vote pour les consultations
const fakeVotes: Record<string, number> = {
  'ch-007': 8,
  'ch-008': 4,
}

const fakeVotesOpen: Record<string, number> = {
  'ch-001': 5,
  'ch-002': 3,
  'ch-003': 2,
  'ch-004': 6,
  'ch-005': 7,
  'ch-006': 3,
}

function totalVotes(choices: { id: string }[], fakeMap: Record<string, number>): number {
  return choices.reduce((s, c) => s + (fakeMap[c.id] ?? 0), 0)
}

function pct(choiceId: string, total: number, fakeMap: Record<string, number>): number {
  if (!total) return 0
  return Math.round(((fakeMap[choiceId] ?? 0) / total) * 100)
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
          <span>{{ t('consultations.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('consultations.create') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container">
      <CMockBanner />

      <div class="view-header">
        <div>
          <h1 class="display-md">
            {{ t('consultations.title') }}
          </h1>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('consultations.openCount', mockConsultations.filter(c => c.status === 'open').length) }}
          </p>
        </div>
      </div>

      <div class="tabs">
        <button
          v-for="opt in [
            { key: 'all', label: t('common.allFem') },
            { key: 'open', label: t('consultations.filters.open') },
            { key: 'closed', label: t('consultations.filters.closed') },
          ]"
          :key="opt.key"
          :class="['tab', { 'tab--active': filter === opt.key }]"
          @click="filter = opt.key as typeof filter"
        >
          {{ opt.label }}
        </button>
      </div>

      <CCard class="legal-note">
        <p class="body-sm">
          <strong>{{ t('consultations.informalNote') }}</strong>
          {{ t('consultations.informalNoteDetail') }}
        </p>
      </CCard>

      <div class="consultation-list">
        <CCard
          v-for="c in consultations"
          :key="c.id"
          class="consultation-card"
        >
          <div class="con-header">
            <CBadge
              :variant="c.status === 'open' ? 'open' : 'closed'"
              dot
            >
              {{ t(`consultations.statuses.${c.status}`) }}
            </CBadge>
            <span class="con-date">
              <span v-if="c.status === 'open'">{{ t('consultations.openedAt', { date: formatRelative(c.created_at) }) }}</span>
              <span v-else>{{ t('consultations.closedAt', { date: formatDate(c.closed_at!) }) }}</span>
            </span>
          </div>

          <h3 class="title-md">
            {{ c.title }}
          </h3>
          <p
            v-if="c.description"
            class="body-sm"
            style="color:var(--color-on-surface-variant)"
          >
            {{ c.description }}
          </p>

          <!-- Résultats (consultation clôturée) -->
          <div
            v-if="c.status === 'closed'"
            class="choices-results"
          >
            <div
              v-for="choice in c.choices"
              :key="choice.id"
              class="choice-result"
            >
              <div class="choice-label">
                <span>{{ choice.label }}</span>
                <span class="choice-pct">{{ pct(choice.id, totalVotes(c.choices, fakeVotes), fakeVotes) }}%</span>
              </div>
              <div class="choice-bar-bg">
                <div
                  class="choice-bar-fill"
                  :style="{ width: pct(choice.id, totalVotes(c.choices, fakeVotes), fakeVotes) + '%' }"
                ></div>
              </div>
            </div>
            <p class="votes-total">
              {{ t('consultations.responseCount', { count: totalVotes(c.choices, fakeVotes) }) }}
            </p>
          </div>

          <!-- Choix (consultation ouverte) -->
          <div
            v-else
            class="choices-open"
          >
            <div
              v-for="choice in c.choices"
              :key="choice.id"
              class="choice-open"
            >
              <div class="choice-label">
                <span>{{ choice.label }}</span>
                <span class="choice-pct dim">{{ fakeVotesOpen[choice.id] ?? 0 }} réponse(s)</span>
              </div>
              <div class="choice-bar-bg">
                <div
                  class="choice-bar-fill choice-bar-fill--open"
                  :style="{ width: pct(choice.id, totalVotes(c.choices, fakeVotesOpen), fakeVotesOpen) + '%' }"
                ></div>
              </div>
            </div>
            <p class="votes-total">
              {{ t('consultations.responseCountOpen', { count: totalVotes(c.choices, fakeVotesOpen) }) }}
            </p>
          </div>
        </CCard>
      </div>
    </div>
  </div>
</template>

<style scoped>
.legal-note { border-left: 3px solid var(--color-warning); }

.consultation-list { display: flex; flex-direction: column; gap: var(--space-4); }
.consultation-card { display: flex; flex-direction: column; gap: var(--space-3); }

.con-header { display: flex; align-items: center; gap: 10px; }
.con-date { font-size: var(--text-xs); color: var(--color-on-surface-variant); }

.choices-results, .choices-open { display: flex; flex-direction: column; gap: 10px; }
.choice-result, .choice-open { display: flex; flex-direction: column; gap: var(--space-1); }

.choice-label { display: flex; justify-content: space-between; font-size: var(--text-sm); }
.choice-pct { font-weight: var(--fw-semibold); }
.dim { color: var(--color-on-surface-variant); }

.choice-bar-bg { height: 8px; background: var(--color-surface-container); border-radius: var(--radius-full); overflow: hidden; }
.choice-bar-fill { height: 100%; background: var(--color-primary); border-radius: var(--radius-full); transition: width 0.4s ease; }
.choice-bar-fill--open { background: var(--color-secondary); }

.votes-total { font-size: var(--text-xs); color: var(--color-on-surface-variant); margin-top: var(--space-1); }
</style>
