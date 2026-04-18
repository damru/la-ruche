<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { MeetingAttendanceEntry, MeetingAttendanceMode, Resolution, VoteResult } from '@/types/domain'

type BadgeVariant = 'open' | 'pending' | 'closed' | 'disabled' | 'valid' | 'err' | 'primary' | 'default'

const props = defineProps<{
  open: boolean
  meetingId: string
  resolution: Resolution | null
  attendingUnits: MeetingAttendanceEntry[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': [payload: { resolutionId: string; result: VoteResult | null; finalized: boolean; eligibleForSecondVote: string | null }]
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const unitVotes = ref<Record<string, string>>({})

const votePositionOptions = computed(() => [
  { value: '', label: t('meetings.vote.notSet') },
  { value: 'in_favor', label: t('meetings.vote.in_favor') },
  { value: 'against', label: t('meetings.vote.against') },
  { value: 'abstain', label: t('meetings.vote.abstain') },
])

function attendanceModeLabel(mode: MeetingAttendanceMode): string {
  return t(`meetings.attendanceModes.${mode}`)
}

function attendanceModeVariant(mode: MeetingAttendanceMode): BadgeVariant {
  if (mode === 'present') return 'valid'
  if (mode === 'proxy') return 'pending'
  if (mode === 'correspondence') return 'primary'
  return 'disabled'
}

function fillAllVotes(position: 'in_favor' | 'against') {
  for (const entry of props.attendingUnits) {
    unitVotes.value[entry.unit_id] = position
  }
}

watch(() => props.open, async (open) => {
  if (!open) { forceClose(); return }
  unitVotes.value = {}
  feedback.reset()
  if (!props.resolution) return
  try {
    const res = await meetingsApi.listVotes(props.meetingId, props.resolution.id)
    for (const v of res.data) {
      unitVotes.value[v.unit_id] = v.position
    }
  } catch {
    // pre-fill failure non-blocking — manager can still enter votes
  }
})

async function handleSubmit() {
  if (!props.resolution) return
  saving.value = true
  feedback.reset()
  try {
    await Promise.all(
      props.attendingUnits
        .filter(e => unitVotes.value[e.unit_id])
        .map(e => meetingsApi.vote(props.meetingId, props.resolution!.id, {
          unit_id: e.unit_id,
          position: unitVotes.value[e.unit_id],
        })),
    )

    const finalizeRes = await meetingsApi.finalizeResolution(
      props.meetingId,
      props.resolution.id,
    )

    if (finalizeRes.data.finalized && finalizeRes.data.result) {
      const msgKey = finalizeRes.data.eligible_for_second_vote === 'art_25_1'
        ? 'meetings.vote.eligibleSecondVote25_1'
        : finalizeRes.data.eligible_for_second_vote === 'art_26_1'
          ? 'meetings.vote.eligibleSecondVote26_1'
          : finalizeRes.data.result.passed
            ? 'meetings.vote.passed'
            : 'meetings.vote.rejected'
      feedback.setSuccess(t(msgKey))
      emit('saved', {
        resolutionId: props.resolution.id,
        result: finalizeRes.data.result,
        finalized: true,
        eligibleForSecondVote: finalizeRes.data.eligible_for_second_vote ?? null,
      })
    } else {
      const resultRes = await meetingsApi.results(props.meetingId, props.resolution.id)
      feedback.setSuccess(t('meetings.vote.voteSaved'))
      emit('saved', {
        resolutionId: props.resolution.id,
        result: resultRes.data,
        finalized: false,
        eligibleForSecondVote: null,
      })
    }

    scheduleClose(() => { emit('update:open', false) })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    size="lg"
    :title="resolution ? t('meetings.vote.enterVotesTitle', { title: resolution.title }) : ''"
    @update:open="$emit('update:open', $event)"
  >
    <p
      v-if="attendingUnits.length === 0"
      class="body-sm tone--muted"
    >
      {{ t('meetings.vote.noAttendees') }}
    </p>
    <template v-else>
      <div class="vote-quick-actions">
        <CBtn
          size="sm"
          variant="ghost"
          :title="t('meetings.vote.fillAllFor')"
          @click="fillAllVotes('in_favor')"
        />
        <CBtn
          size="sm"
          variant="ghost"
          :title="t('meetings.vote.fillAllAgainst')"
          @click="fillAllVotes('against')"
        />
      </div>
      <table class="vote-table">
        <thead>
          <tr>
            <th>{{ t('units.number') }}</th>
            <th>{{ t('units.owner') }}</th>
            <th>{{ t('meetings.participants') }}</th>
            <th>{{ t('meetings.vote.myVote') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="entry in attendingUnits"
            :key="entry.unit_id"
          >
            <td class="font-mono">
              {{ entry.unit_number }}
            </td>
            <td>
              {{ entry.owner_name || '—' }}
            </td>
            <td>
              <CBadge :variant="attendanceModeVariant(entry.mode)">
                {{ attendanceModeLabel(entry.mode) }}
              </CBadge>
            </td>
            <td :class="[!unitVotes[entry.unit_id] && 'vote-table__select--empty']">
              <CFormSelect
                :model-value="unitVotes[entry.unit_id] ?? ''"
                :options="votePositionOptions"
                @update:model-value="unitVotes[entry.unit_id] = $event"
              />
            </td>
          </tr>
        </tbody>
      </table>
    </template>
    <template #footer>
      <CFormActions
        :status="feedback.status.value"
        :message="feedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :title="t('meetings.vote.submit')"
          :loading="saving"
          :disabled="attendingUnits.length === 0"
          @click="handleSubmit"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>
.vote-quick-actions {
  display: flex;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.vote-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--text-sm);
}

.vote-table th {
  text-align: left;
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-xs);
  font-weight: var(--fw-medium);
  color: var(--color-on-surface-variant);
  border-bottom: 1px solid var(--color-outline-variant);
}

.vote-table td {
  padding: var(--space-2) var(--space-3);
  border-bottom: 1px solid var(--color-outline-variant);
  vertical-align: middle;
}

.vote-table tbody tr:last-child td {
  border-bottom: none;
}

.vote-table__select--empty :deep(select) {
  box-shadow: var(--shadow-error-ring);
}
</style>
