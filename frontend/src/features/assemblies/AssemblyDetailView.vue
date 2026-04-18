<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.vue'
import CIconPlus from '@/components/atoms/CIconPlus.vue'
import CIconCheck from '@/components/atoms/CIconCheck.vue'
import CIconCalendar from '@/components/atoms/CIconCalendar.vue'
import CIconLocation from '@/components/atoms/CIconLocation.vue'
import CIconUser from '@/components/atoms/CIconUser.vue'
import CIconDownload from '@/components/atoms/CIconDownload.vue'
import CIconClock from '@/components/atoms/CIconClock.vue'
import CIconAlertTriangle from '@/components/atoms/CIconAlertTriangle.vue'
import { mockMeetings, mockResolutions, mockVoteResults, mockBuildingStats } from '@/mock/data'
import { Teleport } from 'vue'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const { formatDate } = useFormatters()

const meeting = computed(() =>
  mockMeetings.find(m => m.id === route.params.id) ?? mockMeetings[0]
)
const resolutions = computed(() =>
  mockResolutions.filter(r => r.meeting_id === meeting.value.id)
)

const showAddResolutionModal = ref(false)
const showClotureModal = ref(false)
const showPvModal = ref(false)

// Mock participants
const participants = [
  { name: 'Marie Lefebvre', lot: 'Lot 1', tantièmes: 120, color: '#dde1f9', text: '#1a1d2e' },
  { name: 'Robert Cazeneuve', lot: 'Lot 2', tantièmes: 85, color: '#e8f5e9', text: '#1b4f23' },
  { name: 'Claire Renaud', lot: 'Lot 3', tantièmes: 95, color: '#dde1f9', text: '#1a1d2e', mandataire: 'Lefebvre Marie' },
  { name: 'Fatima Bensalem', lot: 'Lot 4', tantièmes: 110, color: '#fce8d5', text: '#7a3600' },
  { name: 'Jean-Pierre Dupont', lot: 'Lot 5', tantièmes: 130, color: '#dde1f9', text: '#1a1d2e' },
]

const sharesPresent = computed(() => participants.reduce((s, p) => s + p.tantièmes, 0))
const totalShares = 1000
const quorumPct = computed(() => Math.round((sharesPresent.value / totalShares) * 100))
const quorumOk = computed(() => sharesPresent.value > totalShares / 4)

// ODJ items = resolution titles
const odjItems = computed(() => resolutions.value.map(r => r.title))

function initials(name: string): string {
  return name.split(' ').map(w => w[0]).join('').toUpperCase().slice(0, 2)
}

function statusLabel(s: string): string {
  return t(`meetings.resolutionStatuses.${s}`)
}
function statusChipClass(s: string): string {
  if (s === 'passed') return 'tone--valid'
  if (s === 'rejected') return 'tone--err'
  return 'tone--disabled'
}
function voteTypeLabel(t_: string): string {
  return t(`meetings.vote.typeShort.${t_}`)
}
function voteTypeClass(t_: string): string {
  if (t_ === 'simple_majority') return 'tone--default'
  if (t_ === 'absolute_majority') return 'chip-art25'
  return 'chip-art26'
}
function resBadgeClass(s: string): string {
  if (s === 'passed') return 'tone--accent'
  return 'tone--disabled'
}
function votePct(votes: number, present: number): number {
  if (!present) return 0
  return Math.round((votes / present) * 100)
}

function meetingStatusLabel(s: string): string {
  return t(`meetings.statuses.${s}`)
}
function meetingStatusChip(s: string): string {
  if (s === 'called') return 'tone--default'
  if (s === 'closed') return 'tone--valid'
  return 'tone--disabled'
}
function typeLabel(type: string): string {
  return t(`meetings.typeLabel.${type === 'annual' ? 'annual' : 'extraordinary'}`)
}
</script>

<template>
  <div class="detail-page">

    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          <span @click="router.push('/')" style="cursor:pointer;color:var(--color-on-surface-variant)">{{ mockBuildingStats.name }}</span>
          <CIconChevronRight width="14" height="14"/>
          <span @click="router.push('/meetings')" style="cursor:pointer;color:var(--color-on-surface-variant)">Assemblées générales</span>
          <CIconChevronRight width="14" height="14"/>
          <span>{{ typeLabel(meeting.meeting_type) }} {{ new Date(meeting.meeting_date).getFullYear() }}</span>
        </div>
      </template>
      <template #right>
        <CBtn variant="secondary" size="sm" @click="showAddResolutionModal = true">
          <CIconPlus/>
          {{ t('meetings.addResolution') }}
        </CBtn>
        <CBtn variant="primary" size="sm" @click="showClotureModal = true">
          <CIconCheck/>
          {{ t('meetings.close') }}
        </CBtn>
      </template>
    </CTopbar>

    <div class="content">
      <CMockBanner />

      <!-- Page header -->
      <div class="page-eyebrow">
        <span :class="['chip', meetingStatusChip(meeting.status)]">
          <span class="chip-dot"></span>
          {{ meetingStatusLabel(meeting.status) }}
        </span>
        {{ t('meetings.assembly') }}
      </div>
      <h1 class="page-title">{{ typeLabel(meeting.meeting_type) }} {{ new Date(meeting.meeting_date).getFullYear() }}</h1>
      <div class="page-meta">
        <span class="meta-item">
          <CIconCalendar/>
          {{ formatDate(meeting.meeting_date) }}
        </span>
        <span class="meta-sep"></span>
        <span class="meta-item">
          <CIconLocation/>
          {{ meeting.location }}
        </span>
        <span class="meta-sep"></span>
        <span class="meta-item">
          <CIconUser/>
          {{ participants.length }} présents · {{ t('meetings.calledAt', { date: meeting.notice_date ? formatDate(meeting.notice_date) : '—' }) }}
        </span>
      </div>

      <!-- Status banner -->
      <div :class="['status-banner', meeting.status === 'closed' ? 'status-banner--closed' : 'status-banner--called']">
        <div class="banner-left">
          <div :class="['banner-icon', meeting.status === 'closed' ? 'banner-icon--closed' : 'banner-icon--called']">
            <CIconCheck v-if="meeting.status === 'closed'"/>
            <CIconCalendar v-else/>
          </div>
          <div>
            <div class="banner-title">
              {{ meeting.status === 'closed' ? t('meetings.bannerClosed') : t('meetings.bannerCalled') }}
            </div>
            <div class="banner-sub">
              {{ t('meetings.resolutionCount', resolutions.length) }}
            </div>
          </div>
        </div>
        <div class="banner-right">
          <CBtn variant="secondary" size="sm" @click="showPvModal = true">
            <CIconDownload/>
            {{ t('meetings.depositMinutes') }}
          </CBtn>
        </div>
      </div>

      <!-- Two-col layout -->
      <div class="layout">

        <!-- LEFT: résolutions -->
        <div class="res-col">
          <div class="res-col-head">
            <div>
              <h2 class="section-title-lg">{{ t('meetings.resolutions') }}</h2>
              <p class="section-sub">{{ t('meetings.resolutionCount', resolutions.length) }}</p>
            </div>
            <button class="btn-ghost-sm" @click="showAddResolutionModal = true">
              <CIconPlus/>
              {{ t('common.add') }}
            </button>
          </div>

          <div class="res-list">
            <div
              v-for="res in resolutions"
              :key="res.id"
              class="res-card"
            >
              <!-- Header -->
              <div class="res-header">
                <div :class="['res-num', resBadgeClass(res.status)]">{{ res.number }}</div>
                <div class="res-meta">
                  <h3 class="res-title">{{ res.title }}</h3>
                  <div class="res-chips">
                    <span :class="['chip', voteTypeClass(res.vote_type)]">{{ voteTypeLabel(res.vote_type) }}</span>
                    <span :class="['chip', statusChipClass(res.status)]">
                      <span class="chip-dot"></span>
                      {{ statusLabel(res.status) }}
                    </span>
                  </div>
                </div>
              </div>

              <!-- Description -->
              <p v-if="res.description" class="res-desc">{{ res.description }}</p>

              <div class="res-divider"></div>

              <!-- Vote section -->
              <div class="vote-section">
                <template v-if="mockVoteResults[res.id]">
                  <div class="vote-title">{{ t('meetings.vote.tally') }}</div>
                  <div class="vote-bars">
                    <div class="vote-row">
                      <span class="vote-label">{{ t('meetings.vote.in_favor') }}</span>
                      <div class="vote-track">
                        <div class="vote-fill vote-fill--for" :style="{ width: votePct(mockVoteResults[res.id].votes_for, mockVoteResults[res.id].shares_present) + '%' }"></div>
                      </div>
                      <span class="vote-pct vote-pct--for">{{ votePct(mockVoteResults[res.id].votes_for, mockVoteResults[res.id].shares_present) }}%</span>
                      <span class="vote-shares">{{ mockVoteResults[res.id].votes_for }} t.</span>
                    </div>
                    <div class="vote-row">
                      <span class="vote-label">{{ t('meetings.vote.against') }}</span>
                      <div class="vote-track">
                        <div class="vote-fill vote-fill--against" :style="{ width: votePct(mockVoteResults[res.id].votes_against, mockVoteResults[res.id].shares_present) + '%' }"></div>
                      </div>
                      <span class="vote-pct vote-pct--against">{{ votePct(mockVoteResults[res.id].votes_against, mockVoteResults[res.id].shares_present) }}%</span>
                      <span class="vote-shares">{{ mockVoteResults[res.id].votes_against }} t.</span>
                    </div>
                    <div class="vote-row">
                      <span class="vote-label">{{ t('meetings.vote.abstain') }}</span>
                      <div class="vote-track">
                        <div class="vote-fill vote-fill--abstain" :style="{ width: votePct(mockVoteResults[res.id].abstentions, mockVoteResults[res.id].shares_present) + '%' }"></div>
                      </div>
                      <span class="vote-pct vote-pct--abstain">{{ votePct(mockVoteResults[res.id].abstentions, mockVoteResults[res.id].shares_present) }}%</span>
                      <span class="vote-shares">{{ mockVoteResults[res.id].abstentions }} t.</span>
                    </div>
                  </div>
                  <div :class="['vote-result', mockVoteResults[res.id].adopted ? 'vote-result--passed' : 'vote-result--rejected']">
                    <div>
                      <div :class="['vote-result-text', mockVoteResults[res.id].adopted ? 'text--ok' : 'text--err']">
                        {{ mockVoteResults[res.id].adopted ? '✓ ' + t('meetings.vote.passed') : '✗ ' + t('meetings.vote.rejected') }}
                      </div>
                      <div class="vote-result-sub">{{ mockVoteResults[res.id].explanation }}</div>
                    </div>
                    <span class="vote-result-count">
                      {{ mockVoteResults[res.id].votes_for }} / {{ mockVoteResults[res.id].shares_present }}
                    </span>
                  </div>
                </template>
                <template v-else>
                  <div class="vote-pending">
                    <CIconClock/>
                    {{ t('meetings.vote.pendingVote') }}
                  </div>
                </template>
              </div>
            </div>
          </div>
        </div>

        <!-- RIGHT: sidebar -->
        <div class="right-col">

          <!-- Infos card -->
          <div class="info-card">
            <h3 class="info-title">{{ t('meetings.info') }}</h3>
            <div class="info-row">
              <span class="info-key">{{ t('meetings.infoFields.type') }}</span>
              <span class="info-val">{{ typeLabel(meeting.meeting_type) }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">{{ t('meetings.infoFields.date') }}</span>
              <span class="info-val">{{ formatDate(meeting.meeting_date) }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">{{ t('meetings.infoFields.location') }}</span>
              <span class="info-val">{{ meeting.location }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">{{ t('meetings.infoFields.notice') }}</span>
              <span class="info-val">{{ meeting.notice_date ? formatDate(meeting.notice_date) : '—' }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">{{ t('meetings.infoFields.resolutions') }}</span>
              <span class="info-val info-val--mono">{{ resolutions.length }}</span>
            </div>
          </div>

          <!-- Présence & quorum -->
          <div class="presence-card">
            <h3 class="presence-title">{{ t('meetings.presenceQuorum') }}</h3>
            <p class="presence-sub">{{ t('meetings.sharesRepresented') }}</p>
            <div class="presence-big">{{ sharesPresent }} <span class="presence-unit">/ {{ totalShares }}</span></div>
            <p class="presence-detail">{{ t('meetings.attendeesDetail', { count: participants.length }) }}</p>
            <div class="prog-track">
              <div class="prog-fill prog-fill--primary" :style="{ width: quorumPct + '%' }"></div>
            </div>
            <div class="presence-legend">
              <span>0</span>
              <span>{{ quorumPct }}% des tantièmes</span>
              <span>1000</span>
            </div>
            <div v-if="quorumOk" class="quorum-alert tone--valid">
              <CIconCheck/>
              {{ t('meetings.quorumOk') }}
            </div>
            <div v-else class="quorum-alert quorum-alert--warn tone--warn">
              <CIconAlertTriangle/>
              {{ t('meetings.quorumKo') }}
            </div>

            <!-- Participants -->
            <div class="part-list">
              <div v-for="p in participants" :key="p.name" class="part-item">
                <div class="part-avatar" :style="{ background: p.color, color: p.text }">{{ initials(p.name) }}</div>
                <div class="part-info">
                  <div class="part-name">{{ p.name }}</div>
                  <div class="part-lot">{{ p.lot }}</div>
                </div>
                <span class="part-shares">{{ p.tantièmes }} t.</span>
                <span v-if="p.mandataire" class="part-power tone--warn">{{ t('meetings.mandate') }}</span>
              </div>
            </div>
          </div>

          <!-- ODJ card -->
          <div class="odj-card">
            <h3 class="info-title">{{ t('meetings.agenda') }}</h3>
            <div class="odj-list">
              <div v-for="(item, i) in odjItems" :key="i" class="odj-item">
                <span class="odj-n">{{ i + 1 }}.</span>
                <span class="odj-text">{{ item }}</span>
              </div>
            </div>
          </div>

        </div><!-- /right-col -->

      </div><!-- /layout -->
    </div><!-- /content -->

    <!-- Modal: Ajouter résolution -->
    <Teleport to="body">
      <div :class="['modal-overlay', { 'modal-overlay--show': showAddResolutionModal }]" @click.self="showAddResolutionModal = false">
        <div class="modal" role="dialog">
          <h2 class="modal-title">{{ t('meetings.newResolution') }}</h2>
          <p class="modal-sub">{{ t('meetings.newResolutionSub') }}</p>
          <div class="field">
            <label>{{ t('meetings.resolutionLabel') }}</label>
            <input type="text" :placeholder="t('meetings.resolutionLabelPlaceholder')" />
          </div>
          <div class="field">
            <label>{{ t('meetings.resolutionDescription') }}</label>
            <textarea :placeholder="t('meetings.resolutionDescriptionPlaceholder')"></textarea>
          </div>
          <div class="field">
            <label>{{ t('meetings.voteRule') }}</label>
            <select>
              <option value="simple_majority">Article 24 — Majorité simple</option>
              <option value="absolute_majority">Article 25 — Majorité absolue</option>
              <option value="double_majority">Article 26 — Double majorité</option>
            </select>
          </div>
          <div class="modal-actions">
            <CBtn variant="secondary" size="sm" @click="showAddResolutionModal = false">{{ t('common.cancel') }}</CBtn>
            <CBtn variant="primary" size="sm" @click="showAddResolutionModal = false">{{ t('common.add') }}</CBtn>
          </div>
        </div>
      </div>

      <!-- Modal: Clôturer AG -->
      <div :class="['modal-overlay', { 'modal-overlay--show': showClotureModal }]" @click.self="showClotureModal = false">
        <div class="modal" role="dialog">
          <h2 class="modal-title">{{ t('meetings.closeMeetingTitle') }}</h2>
          <p class="modal-sub">{{ t('meetings.closeMeetingSub') }}</p>
          <div class="modal-actions">
            <CBtn variant="secondary" size="sm" @click="showClotureModal = false">{{ t('common.cancel') }}</CBtn>
            <CBtn variant="primary" size="sm" @click="showClotureModal = false">{{ t('meetings.closeMeetingConfirm') }}</CBtn>
          </div>
        </div>
      </div>

      <!-- Modal: Déposer PV -->
      <div :class="['modal-overlay', { 'modal-overlay--show': showPvModal }]" @click.self="showPvModal = false">
        <div class="modal" role="dialog">
          <h2 class="modal-title">{{ t('meetings.pvTitle') }}</h2>
          <p class="modal-sub">{{ t('meetings.pvSub') }}</p>
          <div class="field">
            <label>{{ t('meetings.pvFileLabel') }}</label>
            <input type="file" accept=".pdf" style="height:auto;padding:10px 14px" />
          </div>
          <div class="modal-actions">
            <CBtn variant="secondary" size="sm" @click="showPvModal = false">{{ t('common.cancel') }}</CBtn>
            <CBtn variant="primary" size="sm" @click="showPvModal = false">{{ t('meetings.pvSubmit') }}</CBtn>
          </div>
        </div>
      </div>
    </Teleport>

  </div>
</template>

<style scoped>
.detail-page {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: var(--color-surface);
}
.content { padding: 40px; flex: 1; }

/* Header */
.page-eyebrow {
  font-size: 11px;
  font-weight: var(--fw-semibold);
  letter-spacing: .1em;
  text-transform: uppercase;
  color: var(--color-primary);
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}
.page-title {
  font-family: var(--font-display);
  font-size: 36px;
  font-weight: var(--fw-bold);
  font-variation-settings: 'opsz' 72;
  letter-spacing: -.03em;
  color: var(--color-on-surface);
  line-height: 1;
  margin-bottom: 8px;
}
.page-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  margin-bottom: 32px;
}
.meta-item { display: flex; align-items: center; gap: 5px; font-size: 13px; color: var(--color-on-surface-variant); }
.meta-item svg { width: 13px; height: 13px; flex-shrink: 0; }
.meta-sep { width: 3px; height: 3px; border-radius: 50%; background: var(--color-outline); }

/* Status banner */
.status-banner {
  border-radius: var(--radius-lg);
  padding: 16px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 32px;
}
.status-banner--called  { background: var(--color-secondary-container); }
.status-banner--closed  { background: var(--color-success-container, #e8f5e9); }
.banner-left { display: flex; align-items: center; gap: 12px; }
.banner-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.banner-icon svg { width: 20px; height: 20px; }
.banner-icon--called  { background: rgba(100,63,213,.15); color: var(--color-primary); }
.banner-icon--closed  { background: rgba(46,125,50,.15); color: var(--color-success, #2e7d32); }
.banner-title { font-size: 14px; font-weight: var(--fw-semibold); color: var(--color-on-surface); }
.banner-sub { font-size: 12px; color: var(--color-on-surface-variant); margin-top: 2px; }
.banner-right { display: flex; gap: 8px; }

/* Layout */
.layout { display: grid; grid-template-columns: 1fr 320px; gap: 24px; align-items: start; }

/* Résolutions */
.res-col { display: flex; flex-direction: column; gap: 0; }
.res-col-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}
.section-title-lg {
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  letter-spacing: -.01em;
  color: var(--color-on-surface);
}
.section-sub { font-size: 12px; color: var(--color-on-surface-variant); margin-top: 2px; }
.btn-ghost-sm {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: var(--fw-medium);
  border: none;
  cursor: pointer;
  background: transparent;
  color: var(--color-primary);
  font-family: var(--font-body);
  transition: background .2s ease;
}
.btn-ghost-sm:hover { background: var(--color-secondary-container); }
.btn-ghost-sm svg { width: 13px; height: 13px; }

.res-list { display: flex; flex-direction: column; gap: 16px; }
.res-card {
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  overflow: hidden;
  transition: box-shadow .2s ease;
}
.res-card:hover { box-shadow: var(--shadow-lg); }
.res-header { display: flex; align-items: flex-start; gap: 14px; padding: 20px 24px 16px; }
.res-num {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  background: var(--tone-bg);
  color: var(--tone-fg);
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: var(--fw-bold);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.res-meta { flex: 1; min-width: 0; }
.res-title {
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  color: var(--color-on-surface);
  letter-spacing: -.01em;
  margin-bottom: 6px;
}
.res-chips { display: flex; gap: 6px; flex-wrap: wrap; }
.res-desc { padding: 0 24px 16px; font-size: 13px; color: var(--color-on-surface-variant); line-height: 1.6; }
.res-divider { height: 1px; background: rgba(194,199,207,.2); margin: 0 24px; }

/* Vote */
.vote-section { padding: 16px 24px 20px; }
.vote-title {
  font-size: 11px;
  font-weight: var(--fw-semibold);
  letter-spacing: .06em;
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
  margin-bottom: 14px;
}
.vote-bars { display: flex; flex-direction: column; gap: 10px; margin-bottom: 16px; }
.vote-row { display: flex; align-items: center; gap: 10px; }
.vote-label { font-size: 12px; font-weight: var(--fw-medium); width: 70px; flex-shrink: 0; color: var(--color-on-surface-variant); }
.vote-track { flex: 1; height: 8px; border-radius: var(--radius-full); background: var(--color-surface-container-highest); overflow: hidden; }
.vote-fill { height: 100%; border-radius: var(--radius-full); transition: width .8s cubic-bezier(.34,1.3,.64,1); }
.vote-fill--for     { background: linear-gradient(135deg, #1b5e20, #43a047); }
.vote-fill--against { background: linear-gradient(135deg, #ba1a1a, #e53935); }
.vote-fill--abstain { background: var(--color-outline); }
.vote-pct { font-family: var(--font-mono); font-size: 12px; font-weight: var(--fw-medium); width: 40px; text-align: right; flex-shrink: 0; }
.vote-pct--for      { color: var(--color-success, #2e7d32); }
.vote-pct--against  { color: var(--color-error); }
.vote-pct--abstain  { color: var(--color-on-surface-variant); }
.vote-shares { font-family: var(--font-mono); font-size: 11px; color: var(--color-on-surface-variant); width: 70px; text-align: right; flex-shrink: 0; }

.vote-result {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-radius: var(--radius-md);
}
.vote-result--passed   { background: var(--color-success-container, #e8f5e9); }
.vote-result--rejected { background: var(--color-error-container); }
.vote-result-text { font-size: 13px; font-weight: var(--fw-semibold); }
.text--ok { color: var(--color-success, #2e7d32); }
.text--err { color: var(--color-error); }
.vote-result-sub { font-size: 11px; margin-top: 2px; opacity: .8; color: var(--color-on-surface-variant); }
.vote-result-count { font-family: var(--font-mono); font-size: 13px; font-weight: var(--fw-medium); color: var(--color-on-surface); }
.vote-pending {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--color-on-surface-variant);
  padding: 12px 16px;
  background: var(--color-surface-container-low);
  border-radius: var(--radius-md);
}
.vote-pending svg { width: 15px; height: 15px; flex-shrink: 0; }

/* Right column */
.right-col { display: flex; flex-direction: column; gap: 20px; }

/* Info card */
.info-card, .presence-card, .odj-card {
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  padding: 20px;
  box-shadow: var(--shadow-md);
}
.info-title {
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  color: var(--color-on-surface);
  margin-bottom: 16px;
}
.info-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 10px 0;
  border-bottom: 1px solid rgba(194,199,207,.15);
}
.info-row:last-child { border-bottom: none; padding-bottom: 0; }
.info-key { font-size: 12px; color: var(--color-on-surface-variant); }
.info-val { font-size: 13px; font-weight: var(--fw-medium); color: var(--color-on-surface); text-align: right; max-width: 160px; }
.info-val--mono { font-family: var(--font-mono); font-weight: var(--fw-regular); }

/* Présence */
.presence-title {
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  color: var(--color-on-surface);
  margin-bottom: 4px;
}
.presence-sub { font-size: 12px; color: var(--color-on-surface-variant); margin-bottom: 16px; }
.presence-big {
  font-family: var(--font-display);
  font-size: 40px;
  font-weight: var(--fw-bold);
  font-variation-settings: 'opsz' 72;
  letter-spacing: -.03em;
  color: var(--color-on-surface);
  margin-bottom: 2px;
  line-height: 1;
}
.presence-unit { font-size: 20px; color: var(--color-on-surface-variant); }
.presence-detail { font-size: 12px; color: var(--color-on-surface-variant); margin-bottom: 12px; }
.prog-track { height: 10px; border-radius: var(--radius-full); background: var(--color-surface-container-highest); overflow: hidden; margin-bottom: 6px; }
.prog-fill { height: 100%; border-radius: var(--radius-full); transition: width .8s cubic-bezier(.34,1.3,.64,1); }
.prog-fill--primary { background: var(--gradient-primary); }
.presence-legend { display: flex; justify-content: space-between; margin-bottom: 12px; }
.presence-legend span { font-size: 11px; color: var(--color-on-surface-variant); }
.quorum-alert {
  border-radius: var(--radius-sm);
  padding: 10px 12px;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  background: var(--tone-bg);
  color: var(--tone-fg);
}
.quorum-alert svg { width: 14px; height: 14px; flex-shrink: 0; }


/* Participants */
.part-list { display: flex; flex-direction: column; }
.part-item { display: flex; align-items: center; gap: 8px; padding: 8px 0; border-bottom: 1px solid rgba(194,199,207,.12); }
.part-item:last-child { border-bottom: none; }
.part-avatar { width: 28px; height: 28px; border-radius: var(--radius-full); display: flex; align-items: center; justify-content: center; font-family: var(--font-display); font-size: 11px; font-weight: var(--fw-bold); flex-shrink: 0; }
.part-info { flex: 1; min-width: 0; }
.part-name { font-size: 12px; color: var(--color-on-surface); }
.part-lot { font-size: 11px; color: var(--color-on-surface-variant); }
.part-shares { font-family: var(--font-mono); font-size: 11px; color: var(--color-on-surface-variant); }
.part-power { font-size: 10px; padding: 2px 6px; border-radius: var(--radius-full); background: var(--tone-bg); color: var(--tone-fg); font-weight: var(--fw-medium); }

/* ODJ */
.odj-list { display: flex; flex-direction: column; }
.odj-item { display: flex; gap: 10px; padding: 8px 0; border-bottom: 1px solid rgba(194,199,207,.12); }
.odj-item:last-child { border-bottom: none; }
.odj-n { font-family: var(--font-mono); font-size: 11px; color: var(--color-primary); font-weight: var(--fw-medium); width: 20px; flex-shrink: 0; padding-top: 1px; }
.odj-text { font-size: 12px; color: var(--color-on-surface); line-height: 1.4; }

/* Chips */
.chip { display: inline-flex; align-items: center; gap: 5px; padding: 4px 10px; border-radius: var(--radius-full); font-size: 11px; font-weight: var(--fw-medium); background: var(--tone-bg, transparent); color: var(--tone-fg, inherit); }
.chip-dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; opacity: .65; flex-shrink: 0; }
.chip-art25 { --tone-bg: #e8def8; --tone-fg: #21005d; }
.chip-art26 { --tone-bg: #fce8d5; --tone-fg: #7a3600; }

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(25,28,29,.4);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  pointer-events: none;
  transition: opacity .2s;
}
.modal-overlay--show { opacity: 1; pointer-events: all; }
.modal {
  background: rgba(255,255,255,.92);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(194,199,207,.3);
  box-shadow: 0 20px 60px rgba(25,28,29,.14);
  border-radius: var(--radius-lg);
  padding: 28px;
  width: 480px;
  max-width: 90vw;
  transform: translateY(8px) scale(.98);
  transition: transform .25s cubic-bezier(.34,1.3,.64,1);
}
.modal-overlay--show .modal { transform: translateY(0) scale(1); }
.modal-title {
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  letter-spacing: -.01em;
  color: var(--color-on-surface);
  margin-bottom: 6px;
}
.modal-sub { font-size: 13px; color: var(--color-on-surface-variant); margin-bottom: 24px; line-height: 1.5; }
.modal-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 24px; }
.field { margin-bottom: 16px; }
.field label { font-size: 13px; font-weight: var(--fw-medium); color: var(--color-on-surface); display: block; margin-bottom: 6px; }
.field input,
.field select,
.field textarea {
  width: 100%;
  padding: 0 14px;
  height: 42px;
  background: var(--color-surface-container);
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-body);
  font-size: 14px;
  color: var(--color-on-surface);
  outline: none;
  transition: background .2s, box-shadow .2s;
}
.field textarea { height: auto; padding: 12px 14px; resize: vertical; min-height: 72px; }
.field input:focus,
.field select:focus,
.field textarea:focus { background: var(--color-surface-container-low); box-shadow: 0 0 0 2px rgba(100,63,213,.3); }
</style>
