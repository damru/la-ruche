<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CBadge from '@/components/atoms/CBadge.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import { contractsApi } from '@/api'
import { useFormatters } from '@/composables/useFormatters'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { ContractStatus, VendorContract } from '@/types/domain'

const props = defineProps<{
  open: boolean
  contract: VendorContract | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const { formatCurrency, formatDate } = useFormatters()

const loading = ref(false)
const error = ref('')
const chain = ref<VendorContract[]>([])

// Maps a contract status onto the CBadge status variant used across the
// contracts feature (see ContractsView.statusVariant).
function statusVariant(status: ContractStatus): 'open' | 'pending' | 'closed' | 'default' {
  if (status === 'active') return 'open'
  if (status === 'draft') return 'pending'
  if (status === 'terminated' || status === 'expired') return 'closed'
  return 'default'
}

function chainLabel(index: number): string {
  return index === 0
    ? t('contracts.chainOrigin')
    : t('contracts.chainAmendment', { index })
}

async function loadChain(contractId: string) {
  loading.value = true
  error.value = ''
  chain.value = []

  try {
    const response = await contractsApi.amendmentChain(contractId)
    chain.value = response.data
  } catch (currentError: unknown) {
    error.value = toErrorMessage(currentError, t('contracts.historyError'))
  } finally {
    loading.value = false
  }
}

watch(
  () => [props.open, props.contract] as const,
  ([open, contract]) => {
    if (open && contract?.id) {
      void loadChain(contract.id)
    }
  },
  { immediate: true },
)
</script>

<template>
  <CModal
    :open="open"
    size="md"
    :title="t('contracts.historyTitle', { name: contract?.vendor_name ?? '' })"
    @update:open="emit('update:open', $event)"
  >
    <CPageLoader v-if="loading" />
    <p
      v-else-if="error"
      class="body-sm chain__error"
    >
      {{ error }}
    </p>
    <p
      v-else-if="chain.length === 0"
      class="body-sm chain__empty"
    >
      {{ t('contracts.historyEmpty') }}
    </p>
    <ol
      v-else
      class="chain"
    >
      <li
        v-for="(link, index) in chain"
        :key="link.id"
        class="chain__item"
        :class="{ 'chain__item--current': link.id === contract?.id }"
      >
        <div
          class="chain__marker"
          aria-hidden="true"
        ></div>
        <div class="chain__body">
          <div class="chain__head">
            <span class="chain__label">{{ chainLabel(index) }}</span>
            <CBadge
              :variant="statusVariant(link.status)"
              dot
            >
              {{ t(`contracts.statuses.${link.status}`) }}
            </CBadge>
            <CBadge
              v-if="link.id === contract?.id"
              variant="default"
            >
              {{ t('contracts.historyCurrent') }}
            </CBadge>
          </div>
          <div class="chain__meta">
            <span>{{ formatDate(link.start_date) }} → {{ formatDate(link.end_date) }}</span>
            <span>·</span>
            <span>{{ formatCurrency(link.amount) }}</span>
          </div>
        </div>
      </li>
    </ol>
  </CModal>
</template>

<style scoped>
.chain {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.chain__item {
  display: flex;
  gap: var(--space-3);
  align-items: flex-start;
}

.chain__marker {
  margin-top: 6px;
  width: 10px;
  height: 10px;
  border-radius: var(--radius-full);
  background: var(--color-outline-variant);
  flex-shrink: 0;
}

.chain__item--current .chain__marker {
  background: var(--color-primary);
}

.chain__body {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  min-width: 0;
}

.chain__head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-2);
}

.chain__label {
  font-weight: var(--fw-medium);
}

.chain__meta {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-1);
  color: var(--color-on-surface-variant);
  font-size: var(--text-sm);
}

.chain__error {
  color: var(--color-error);
}

.chain__empty {
  color: var(--color-on-surface-variant);
}
</style>
