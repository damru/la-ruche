<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { usePageLoader } from '@/composables/usePageLoader'
import { useAuthStore } from '@/stores/auth'
import CBadge from '@/components/atoms/CBadge.vue'
import CIconTile from '@/components/atoms/CIconTile.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CCardActions from '@/components/molecules/CCardActions.vue'
import CDataTable, { type Column } from '@/components/molecules/CDataTable.vue'
import CFormMultiSelect from '@/components/atoms/CFormMultiSelect.vue'
import CEmptyState from '@/components/atoms/CEmptyState.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CIconBolt from '@/components/atoms/CIconBolt.svg'
import CIconDroplet from '@/components/atoms/CIconDroplet.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconLayers from '@/components/atoms/CIconLayers.svg'
import CIconLeaf from '@/components/atoms/CIconLeaf.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconSearch from '@/components/atoms/CIconSearch.svg'
import CIconSettings from '@/components/atoms/CIconSettings.svg'
import CIconShield from '@/components/atoms/CIconShield.svg'
import CIconSparkles from '@/components/atoms/CIconSparkles.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconUsers from '@/components/atoms/CIconUsers.svg'
import CIconWrench from '@/components/atoms/CIconWrench.svg'
import CIconX from '@/components/atoms/CIconX.svg'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import ContractorFormModal from '@/features/contractors/ContractorFormModal.vue'
import DeleteContractorModal from '@/features/contractors/DeleteContractorModal.vue'
import { contractorsApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import type { Contractor } from '@/types/domain'

const DOMAINS_RAW = ['maintenance', 'insurance', 'service', 'energy', 'water', 'cleaning', 'green_spaces', 'other']

function domainIcon(domains: string[]) {
  const icons: Record<string, unknown> = {
    maintenance: CIconWrench,
    insurance: CIconShield,
    service: CIconSettings,
    energy: CIconBolt,
    water: CIconDroplet,
    cleaning: CIconSparkles,
    green_spaces: CIconLeaf,
    other: CIconLayers,
  }
  // Use the icon of the first domain, fall back to layers
  return icons[domains[0] ?? ''] ?? CIconLayers
}

function domainTone(domains: string[]): string {
  const tones: Record<string, string> = {
    maintenance: 'tone--warn',
    insurance: 'tone--err',
    service: 'tone--default',
    energy: 'tone--accent',
    water: 'tone--default',
    cleaning: 'tone--valid',
    green_spaces: 'tone--valid',
    other: 'tone--default',
  }
  return tones[domains[0] ?? ''] ?? 'tone--default'
}

const { t } = useI18n()
const router = useRouter()
const authStore = useAuthStore()

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const allContractors = ref<Contractor[]>([])
const search = ref('')
const viewMode = ref<'list' | 'grid'>('list')
const selectedDomains = ref<string[]>([])

// ── Permissions ────────────────────────────────────────────────────
const canEdit = computed(() =>
  authStore.hasAnyRole(['manager', 'board_member']),
)

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [building, contractorsRes] = await Promise.all([
    useBuildingStore().fetch(),
    contractorsApi.list(),
  ])
  buildingName.value = building.name
  allContractors.value = contractorsRes.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

// ── Sorted domains list (alphabetical by translated label) ────────
const domainOptions = computed(() =>
  [...DOMAINS_RAW]
    .map(value => ({ value, label: t(`contractors.domains.${value}`) }))
    .sort((a, b) => a.label.localeCompare(b.label)),
)

const hasActiveFilters = computed(() => selectedDomains.value.length > 0)

function clearFilters() {
  selectedDomains.value = []
}

// ── Table columns (list view) ─────────────────────────────────────
const contractorColumns = computed<Column<Contractor>[]>(() => [
  { key: 'name', label: t('lastName') },
  { key: 'domains', label: t('contractors.domainsLabel') },
  { key: 'email', label: t('common.email') },
  { key: 'phone', label: t('common.phone') },
  { key: 'actions', label: '', width: '1%' },
])

// ── Filtered list ─────────────────────────────────────────────────
const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  return allContractors.value.filter((c) => {
    if (selectedDomains.value.length && !c.domains.some(d => selectedDomains.value.includes(d)))
      return false
    if (!q) return true
    return (
      c.name.toLowerCase().includes(q) ||
      (c.email ?? '').toLowerCase().includes(q) ||
      c.domains.some(d =>
        d.toLowerCase().includes(q) ||
        t(`contractors.domains.${d}`).toLowerCase().includes(q),
      )
    )
  })
})

function openContractorDetail(contractorId: string) {
  void router.push({ name: 'contractor-detail', params: { id: contractorId } })
}

// ── Form modal ────────────────────────────────────────────────────
const showModal = ref(false)
const editingContractor = ref<Contractor | null>(null)

function openCreate() {
  editingContractor.value = null
  showModal.value = true
}

function openEdit(c: Contractor) {
  editingContractor.value = c
  showModal.value = true
}

// ── Delete ────────────────────────────────────────────────────────
const showDeleteModal = ref(false)
const contractorToDelete = ref<Contractor | null>(null)

function openDeleteModal(c: Contractor) {
  contractorToDelete.value = c
  showDeleteModal.value = true
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight class="icon-xs" />
          <span>{{ t('contractors.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canEdit"
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('contractors.newContractor')"
          @click="openCreate"
        />
      </template>
    </CTopbar>

    <div class="view-container">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else>
        <header class="view-header">
          <CEyebrow>{{ t('page.eyebrows.contractors') }}</CEyebrow>
          <CHeading type="header">
            {{ t('contractors.title') }}
          </CHeading>
          <CMetaItem variant="sub">
            {{ t('contractors.summary', { count: allContractors.length }) }}
          </CMetaItem>
        </header>

        <!-- Toolbar: search + view mode toggle -->
        <div class="list-toolbar">
          <CFormInput
            v-model="search"
            type="search"
            :placeholder="t('contractors.searchPlaceholder')"
            class="list-toolbar__search"
          >
            <template #prefix>
              <CIconSearch class="icon-sm" />
            </template>
          </CFormInput>
          <div
            class="view-toggle"
            role="group"
            :aria-label="t('common.viewMode', 'Vue')"
          >
            <button
              :class="['view-toggle__btn', { 'view-toggle__btn--active': viewMode === 'list' }]"
              :aria-pressed="viewMode === 'list'"
              @click="viewMode = 'list'"
            >
              <CIconList class="icon-sm" />
            </button>
            <button
              :class="['view-toggle__btn', { 'view-toggle__btn--active': viewMode === 'grid' }]"
              :aria-pressed="viewMode === 'grid'"
              @click="viewMode = 'grid'"
            >
              <CIconGrid class="icon-sm" />
            </button>
          </div>
        </div>

        <!-- Domain filter -->
        <div class="contractors-toolbar">
          <CFormMultiSelect
            v-model="selectedDomains"
            :options="domainOptions"
            :placeholder="t('contractors.domainsLabel')"
            :aria-label="t('contractors.domainsLabel')"
          />
          <CBtn
            v-if="hasActiveFilters"
            variant="ghost"
            size="sm"
            :icon="CIconX"
            :title="t('common.clearFilters')"
            :aria-label="t('common.clearFilters')"
            @click="clearFilters"
          />
        </div>

        <!-- Empty state -->
        <CEmptyState
          v-if="filtered.length === 0"
          :title="search || selectedDomains.length ? t('contractors.noResults') : t('contractors.noContractors')"
          size="page"
        >
          <template #icon>
            <CIconUsers />
          </template>
          <template
            v-if="canEdit && !search && !selectedDomains.length"
            #actions
          >
            <CBtn
              variant="primary"
              size="sm"
              :icon="CIconPlus"
              :title="t('contractors.newContractor')"
              @click="openCreate"
            />
          </template>
        </CEmptyState>

        <!-- ── List view ─────────────────────────────────────────────── -->
        <CCard
          v-else-if="viewMode === 'list'"
          class="p-0!"
        >
          <CDataTable
            :columns="contractorColumns"
            :rows="filtered"
            row-key="id"
            @row-click="openContractorDetail($event.id)"
          >
            <template #cell-name="{ row }">
              <div class="flex items-center gap-3 min-w-0">
                <CIconTile :class="domainTone(row.domains)">
                  <component
                    :is="domainIcon(row.domains)"
                    aria-hidden="true"
                  />
                </CIconTile>
                <span class="font-medium text-on-surface truncate">{{ row.name }}</span>
              </div>
            </template>
            <template #cell-domains="{ row }">
              <div class="flex flex-wrap gap-x-2 gap-y-1">
                <CBadge
                  v-for="d in row.domains.slice(0, 2)"
                  :key="d"
                  variant="default"
                >
                  {{ t(`contractors.domains.${d}`, d) }}
                </CBadge>
                <CBadge
                  v-if="row.domains.length > 2"
                  variant="default"
                >
                  +{{ row.domains.length - 2 }}
                </CBadge>
              </div>
            </template>
            <template #cell-email="{ row }">
              <a
                v-if="row.email"
                :href="`mailto:${row.email}`"
                class="text-xs text-primary no-underline truncate hover:underline"
                @click.stop
              >{{ row.email }}</a>
              <span
                v-else
                class="text-on-surface-variant"
              >—</span>
            </template>
            <template #cell-phone="{ row }">
              <a
                v-if="row.phone"
                :href="`tel:${row.phone}`"
                class="text-xs text-primary no-underline whitespace-nowrap hover:underline"
                @click.stop
              >{{ row.phone }}</a>
              <span
                v-else
                class="text-on-surface-variant"
              >—</span>
            </template>
            <template #cell-actions="{ row }">
              <div
                v-if="canEdit"
                class="flex gap-1 justify-end"
              >
                <CBtn
                  variant="icon"
                  :icon="CIconEdit"
                  :title="t('common.edit')"
                  :aria-label="`${t('common.edit')} ${row.name}`"
                  @click.stop="openEdit(row)"
                />
                <CBtn
                  variant="icon"
                  :icon="CIconTrash"
                  :title="t('common.delete')"
                  :aria-label="`${t('common.delete')} ${row.name}`"
                  class="tone--err"
                  @click.stop="openDeleteModal(row)"
                />
              </div>
            </template>
          </CDataTable>
        </CCard>

        <!-- ── Grid view ─────────────────────────────────────────────── -->
        <div
          v-else
          class="contractors-grid"
        >
          <CCard
            v-for="c in filtered"
            :key="c.id"
            interactive
            gap="sm"
            tabindex="-1"
            @click="openContractorDetail(c.id)"
          >
            <div class="flex items-center gap-3 min-w-0">
              <CIconTile :class="domainTone(c.domains)">
                <component
                  :is="domainIcon(c.domains)"
                  aria-hidden="true"
                />
              </CIconTile>
              <button
                class="flex-1 min-w-0 overflow-hidden text-ellipsis bg-transparent border-none p-0 text-base font-semibold text-on-surface text-left cursor-pointer leading-snug hover:text-primary focus-visible:text-primary focus-visible:outline-none"
                @click.stop="openContractorDetail(c.id)"
              >
                {{ c.name }}
              </button>
            </div>
            <div class="flex flex-wrap gap-x-2 gap-y-1">
              <CBadge
                v-for="d in c.domains.slice(0, 2)"
                :key="d"
                variant="default"
              >
                {{ t(`contractors.domains.${d}`, d) }}
              </CBadge>
              <CBadge
                v-if="c.domains.length > 2"
                variant="default"
              >
                +{{ c.domains.length - 2 }}
              </CBadge>
            </div>
            <div class="flex flex-col gap-3">
              <p
                v-if="c.notes"
                class="body-xs"
              >
                {{ c.notes }}
              </p>
              <div
                v-if="c.email || c.phone"
                class="flex flex-col gap-1"
              >
                <a
                  v-if="c.email"
                  :href="`mailto:${c.email}`"
                  class="text-xs text-primary no-underline truncate hover:underline"
                  @click.stop
                >{{ c.email }}</a>
                <a
                  v-if="c.phone"
                  :href="`tel:${c.phone}`"
                  class="text-xs text-primary no-underline truncate hover:underline"
                  @click.stop
                >{{ c.phone }}</a>
              </div>
              <CCardActions v-if="canEdit">
                <CBtn
                  variant="icon"
                  :icon="CIconEdit"
                  :title="t('common.edit')"
                  :aria-label="`${t('common.edit')} ${c.name}`"
                  @click.stop="openEdit(c)"
                />
                <CBtn
                  variant="icon"
                  :icon="CIconTrash"
                  :title="t('common.delete')"
                  :aria-label="`${t('common.delete')} ${c.name}`"
                  class="tone--err"
                  @click.stop="openDeleteModal(c)"
                />
              </CCardActions>
            </div>
          </CCard>
        </div>
      </template>
    </div>

    <ContractorFormModal
      :open="showModal"
      :contractor="editingContractor"
      @update:open="showModal = $event"
      @saved="reload"
    />

    <DeleteContractorModal
      :open="showDeleteModal"
      :contractor="contractorToDelete"
      @update:open="showDeleteModal = $event"
      @deleted="reload"
    />
  </div>
</template>

<style scoped>
/* ── Toolbar ─────────────────────────────────────────────────────── */
.list-toolbar {
  display: flex;
  gap: var(--space-3);
  align-items: center;
}

.list-toolbar__search {
  flex: 1;
}

.view-toggle {
  display: flex;
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-xl);
  overflow: hidden;
  flex-shrink: 0;
}

.view-toggle__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: var(--color-surface);
  border: none;
  cursor: pointer;
  color: var(--color-on-surface-variant);
  transition: background 150ms ease-out, color 150ms ease-out;
}

.view-toggle__btn + .view-toggle__btn {
  border-left: 1px solid var(--color-outline-variant);
}

.view-toggle__btn:hover {
  background: var(--color-surface-container-low);
  color: var(--color-on-surface);
}

.view-toggle__btn--active {
  background: var(--color-surface-container);
  color: var(--color-on-surface);
}

/* ── Toolbar ─────────────────────────────────────────────────────────────── */
.contractors-toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-2); margin-bottom: var(--space-4); }

/* ── Grid view ───────────────────────────────────────────────────── */
.contractors-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--space-4);
}

</style>
