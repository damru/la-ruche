<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CCard from '@/components/molecules/CCard.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import { buildingApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import type { RegistryResponse, RegistryKey } from '@/types/domain'

const { t } = useI18n()

const data = ref<RegistryResponse | null>(null)

async function loadRegistry() {
  const res = await buildingApi.registry()
  data.value = res.data
}

const { loading, error, reload } = usePageLoader(loadRegistry, t('common.loadError'))
onMounted(reload)

// Build a matrix: unit_id → key_id → share_value
const sharesMatrix = computed(() => {
  if (!data.value) return new Map<string, Map<string, number>>()
  const matrix = new Map<string, Map<string, number>>()
  for (const key of data.value.keys) {
    for (const share of key.shares) {
      if (!matrix.has(share.unit_id)) matrix.set(share.unit_id, new Map())
      matrix.get(share.unit_id)!.set(key.id, share.share_value)
    }
  }
  return matrix
})

// Sorted units for display
const sortedUnits = computed(() =>
  [...(data.value?.units ?? [])].sort((a, b) =>
    a.unit_number.localeCompare(b.unit_number, undefined, { numeric: true })
  )
)

const legalCardLinks = computed(() => [
  {
    label: t('registry.legalCard.linkLabel'),
    href: t('registry.legalCard.linkHref'),
  },
])

function unitTypeIcon(type: string): string {
  const icons: Record<string, string> = { apartment: '🏠', cellar: '🗄️', parking: '🚗', commercial_unit: '🏪', other: '📦' }
  return icons[type] ?? '📦'
}

function isSumMismatch(key: RegistryKey): boolean {
  if (!data.value) return false
  const total = data.value.building.shares_total
  const sum = key.shares_sum
  return total > 0 && Math.abs(sum - total) > 0.001
}

// Format staircase + floor with abbreviated labels, no spurious separator
function formatLocation(unit: { block?: string | null; staircase?: string | null; floor?: string | null }): string {
  const parts: string[] = []
  if (unit.block) parts.push(t('residents.blockShort', { n: unit.block }))
  if (unit.staircase) parts.push(t('residents.staircaseShort', { n: unit.staircase }))
  if (unit.floor) parts.push(t('residents.floorShort', { n: unit.floor }))
  return parts.join(' · ')
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ data?.building.name ?? '...' }}
          <CIconChevronRight />
          <span>{{ t('registry.title') }}</span>
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
          <CEyebrow>
            {{ t('page.eyebrows.registry') }}
          </CEyebrow>
          <h1 class="display-md">
            {{ t('registry.title') }}
          </h1>
          <p
            v-if="data"
            class="body-md dim"
          >
            {{ data.building.address_line1 }}, {{ data.building.postal_code }} {{ data.building.city }}
            · {{ t('registry.totalShares') }} : {{ data.building.shares_total }}
          </p>
        </header>
        <!-- Summary cards -->
        <div class="summary-row">
          <CCard>
            <div class="summary-stat">
              <span class="summary-value">{{ data.units.length }}</span>
              <span class="summary-label">{{ t('registry.lots') }}</span>
            </div>
          </CCard>
          <CCard>
            <div class="summary-stat">
              <span class="summary-value">{{ data.keys.length }}</span>
              <span class="summary-label">{{ t('registry.keys') }}</span>
            </div>
          </CCard>
          <CCard>
            <div class="summary-stat">
              <span class="summary-value">{{ data.building.shares_total }}</span>
              <span class="summary-label">{{ t('registry.totalShares') }}</span>
            </div>
          </CCard>
        </div>

        <!-- Per-key alerts -->
        <CAlert
          v-for="key in data.keys.filter(k => isSumMismatch(k))"
          :key="key.id"
          variant="warning"
        >
          {{ t('registry.keyMismatch', { key: key.name, sum: key.shares_sum, total: data.building.shares_total }) }}
        </CAlert>

        <!-- Cross-table: units × keys -->
        <CCard padding="none">
          <table class="data-table">
            <thead>
              <tr>
                <th class="col-shrink">
                  {{ t('common.lot') }}
                </th>
                <th class="col-shrink">
                  {{ t('common.type') }}
                </th>
                <th class="col-shrink">
                  {{ t('residents.location') }}
                </th>
                <th
                  v-for="key in data.keys"
                  :key="key.id"
                  class="col-right"
                >
                  {{ key.name }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="unit in sortedUnits"
                :key="unit.id"
              >
                <td class="col-shrink">
                  <strong class="data-md">{{ unit.unit_number }}</strong>
                </td>
                <td class="col-shrink">
                  {{ unitTypeIcon(unit.unit_type) }} {{ t(`units.types.${unit.unit_type}`) }}
                </td>
                <td class="col-shrink dim">
                  {{ formatLocation(unit) }}
                </td>
                <td
                  v-for="key in data.keys"
                  :key="key.id"
                  class="cell-numeric"
                >
                  {{ sharesMatrix.get(unit.id)?.get(key.id) ?? '—' }}
                </td>
              </tr>
            </tbody>
            <tfoot>
              <tr class="row-total">
                <td
                  colspan="3"
                  style="font-weight:600"
                >
                  {{ t('common.total') }}
                </td>
                <td
                  v-for="key in data.keys"
                  :key="key.id"
                  class="cell-numeric"
                >
                  <span
                    :class="{ 'sum-mismatch': isSumMismatch(key) }"
                    style="font-weight:600"
                  >
                    {{ key.shares_sum }}
                  </span>
                  <span class="dim">/{{ data.building.shares_total }}</span>
                </td>
              </tr>
            </tfoot>
          </table>
        </CCard>

        <CLegalCard>
          <CLegalNotice :links="legalCardLinks">
            <CLegalDescription
              :title="t('registry.legalCard.title')"
              :summary="t('registry.legalCard.summary')"
            />
          </CLegalNotice>
        </CLegalCard>
      </template>
    </div>
  </div>
</template>

<style scoped>
.dim { color: var(--color-on-surface-variant); }

.summary-row { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-4); }
.summary-stat { display: flex; flex-direction: column; align-items: center; gap: var(--space-1); }
.summary-value { font-size: 28px; font-weight: var(--fw-bold); font-family: var(--font-display); }
.summary-label { font-size: var(--text-sm); color: var(--color-on-surface-variant); }

.sum-mismatch { color: var(--color-error); }
</style>
