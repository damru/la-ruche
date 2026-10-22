<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CEmptyState from '@/components/atoms/CEmptyState.vue'
import { allocationKeysApi, unitsApi } from '@/api'
import type { AllocationKey, AuditEntry, Unit } from '@/types/domain'

const props = defineProps<{
  open: boolean
  keyToShow: AllocationKey | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const loading = ref(false)
const error = ref('')
const entries = ref<AuditEntry[]>([])
const units = ref<Unit[]>([])

function actionLabel(operation: string) {
  const key = `settings.keys.historyActions.${operation}`
  const translated = t(key)
  return translated === key ? operation : translated
}

function justificationOf(entry: AuditEntry) {
  return (entry.new_values?.justification as string | undefined)
    ?? (entry.old_values?.justification as string | undefined)
    ?? '—'
}

function unitLabel(unitId: string) {
  return units.value.find(u => u.id === unitId)?.unit_number ?? unitId
}

function detailsOf(entry: AuditEntry): string {
  const newValues = entry.new_values ?? {}
  const oldValues = entry.old_values ?? {}
  if (entry.operation === 'allocation_key.share_set') {
    const unitId = newValues.unit_id as string | undefined
    const newValue = newValues.share_value as number | undefined
    const oldValue = oldValues.share_value as number | undefined
    if (unitId === undefined || newValue === undefined) return '—'
    return t('settings.keys.historyShareChange', {
      unit: unitLabel(unitId),
      oldValue: oldValue ?? 0,
      newValue,
    })
  }
  if (entry.operation === 'allocation_key.updated') {
    return t('settings.keys.historyNameChange', {
      oldName: (oldValues.name as string | undefined) ?? '—',
      newName: (newValues.name as string | undefined) ?? '—',
    })
  }
  const name = (newValues.name as string | undefined) ?? (oldValues.name as string | undefined)
  return name ?? '—'
}

watch(() => props.open, async (open) => {
  if (!open || !props.keyToShow) return
  loading.value = true
  error.value = ''
  entries.value = []
  try {
    const [auditRes, unitsRes] = await Promise.all([
      allocationKeysApi.listAudit(props.keyToShow.id),
      unitsApi.list(),
    ])
    entries.value = auditRes.data
    units.value = unitsRes.data
  } catch {
    error.value = t('common.error')
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <CModal
    :open="open"
    size="lg"
    :title="t('settings.keys.historyTitle', { name: keyToShow?.name ?? '' })"
    @update:open="emit('update:open', $event)"
  >
    <CPageError
      v-if="error"
      :message="error"
    />
    <CPageLoader v-else-if="loading" />
    <template v-else>
      <CEmptyState
        v-if="entries.length === 0"
        :title="t('settings.keys.historyEmpty')"
      />
      <ul
        v-else
        class="history-list"
      >
        <li
          v-for="entry in entries"
          :key="entry.id"
          class="history-entry"
        >
          <div class="history-entry-header">
            <strong>{{ actionLabel(entry.operation) }}</strong>
            <span class="dim">{{ new Date(entry.changed_at).toLocaleString() }}</span>
          </div>
          <p>
            {{ detailsOf(entry) }}
          </p>
          <p
            v-if="justificationOf(entry) !== '—'"
            class="dim"
          >
            {{ justificationOf(entry) }}
          </p>
        </li>
      </ul>
    </template>

    <template #footer>
      <CBtn
        variant="ghost"
        :title="t('common.close')"
        @click="emit('update:open', false)"
      />
    </template>
  </CModal>
</template>

<style scoped>
.history-list { display: flex; flex-direction: column; gap: var(--space-3); }
.history-entry { border-bottom: 1px solid var(--color-border); padding-bottom: var(--space-2); }
.history-entry-header { display: flex; justify-content: space-between; gap: var(--space-2); }
.dim { color: var(--color-on-surface-variant); font-size: var(--text-sm); }
</style>
