<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CCard from '@/components/molecules/CCard.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import { buildingApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import type { RegistryResponse, RegistryKey } from '@/types/domain'
import router from '@/router'

const { t } = useI18n()

const data = ref<RegistryResponse | null>(null)

async function loadRegistry() {
  const res = await buildingApi.registry()
  data.value = res.data
}

const { loading, error, reload } = usePageLoader(loadRegistry)
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

const registryColumns = computed(() => {
  if (!data.value) return []
  return [
    { key: 'unit_number', label: t('common.lot') },
    { key: 'unit_type', label: t('common.type') },
    { key: 'location', label: t('residents.location') },
    ...data.value.keys.map(k => ({
      key: `key_${k.id}`,
      label: k.name,
      numeric: true,
    })),
  ]
})
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
          <CHeading type="header">
            {{ t('registry.title') }}
          </CHeading>
          <CMetaLine>
            <CMetaItem variant="sub">
              {{ data.building.address_line1 }}, {{ data.building.postal_code }} {{ data.building.city }}
            </CMetaItem>
          </CMetaLine>
        </header>
        
        <div class="grid gap-4 xs:grid-cols-3">
          <CKpiCard
            class="items-center text-center"
            :subtitle="t('common.units')"
            :value="data.units.length.toString()"
          />
          <CKpiCard
            class="items-center text-center"
            :subtitle="t('common.allocationKeys')"
            :value="data.keys.length.toString()"
          />
          <CKpiCard
            class="items-center text-center"
            :subtitle="t('common.totalShares')"
            :value="data.building.shares_total.toString()"
          />
        </div>

        <CAlert
          v-for="key in data.keys.filter(k => isSumMismatch(k))"
          :key="key.id"
          variant="warning"
          dismissible
        >
          {{ t('registry.keyMismatch', { key: key.name, sum: key.shares_sum, total: data.building.shares_total }) }}
          <br /> 
          <CBtn
            size="xs"
            variant="negative"
            @click="router.push({ name: 'residence-keys' })"
          >
            {{ t('common.allocationKeys') }}
            <CIconArrowRight
              class="icon-sm"
              aria-hidden="true"
            />
          </CBtn>
          <template #side>
          </template>
        </CAlert>

        <CCard class="p-0!">
          <CDataTable
            :columns="registryColumns"
            :rows="sortedUnits"
            row-key="id"
          >
            <template #cell-unit_number="{ row }">
              <strong class="data-md">{{ row.unit_number }}</strong>
            </template>
            <template #cell-unit_type="{ row }">
              {{ unitTypeIcon(row.unit_type) }} {{ t(`units.types.${row.unit_type}`) }}
            </template>
            <template #cell-location="{ row }">
              <span class="dim">{{ formatLocation(row) }}</span>
            </template>
            <template
              v-for="k in (data?.keys ?? [])"
              :key="k.id"
              #[`cell-key_${k.id}`]="{ row }"
            >
              {{ sharesMatrix.get(row.id)?.get(k.id) ?? '—' }}
            </template>
            <template #foot-unit_number>
              {{ t('common.total') }}
            </template>
            <template
              v-for="k in (data?.keys ?? [])"
              :key="k.id"
              #[`foot-key_${k.id}`]
            >
              <span :class="{ 'text-error': isSumMismatch(k) }">{{ k.shares_sum }}</span>
              <span class="text-on-surface-variant">/{{ data.building.shares_total }}</span>
            </template>
          </CDataTable>
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
