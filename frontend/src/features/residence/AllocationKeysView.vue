<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconClock from '@/components/atoms/CIconClock.svg'
import { allocationKeysApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import KeyFormModal from '@/features/residence/KeyFormModal.vue'
import DeleteKeyModal from '@/features/residence/DeleteKeyModal.vue'
import KeyAuditModal from '@/features/residence/KeyAuditModal.vue'
import EddTab from '@/features/residence/EddTab.vue'
import type { AllocationKey } from '@/types/domain'

const { t } = useI18n()

const activeTab = ref('keys')
const tabs = computed(() => [
  { value: 'keys', label: t('allocationKeys.tabs.keys') },
  { value: 'edd', label: t('allocationKeys.tabs.edd') },
])

const eddTabRef = ref<InstanceType<typeof EddTab> | null>(null)

function openEddForm() {
  eddTabRef.value?.openForm()
}

const keys = ref<AllocationKey[]>([])

type KeyBalance = { shares_sum: number; shares_total: number; delta: number }
const keyBalance = ref<Record<string, KeyBalance>>({})

const columns = computed(() => [
  { key: 'name', label: t('common.name') },
  { key: 'description', label: t('settings.keys.descriptionLabel') },
  { key: 'shares', label: t('units.shares'), numeric: true },
  { key: 'balance', label: t('common.status') },
  { key: 'actions', label: '', width: '1%' },
])

// ── Modal state ───────────────────────────────────────────────────
const showKeyModal = ref(false)
const keyToEdit = ref<AllocationKey | null>(null)
const showDeleteKeyModal = ref(false)
const keyToDelete = ref<AllocationKey | null>(null)
const showAuditModal = ref(false)
const keyToShowAudit = ref<AllocationKey | null>(null)

function openCreateModal() {
  keyToEdit.value = null
  showKeyModal.value = true
}

function openEditModal(key: AllocationKey) {
  keyToEdit.value = key
  showKeyModal.value = true
}

function openDeleteKeyModal(key: AllocationKey) {
  keyToDelete.value = key
  showDeleteKeyModal.value = true
}

function openAuditModal(key: AllocationKey) {
  keyToShowAudit.value = key
  showAuditModal.value = true
}

async function handleKeySaved(key: AllocationKey) {
  const idx = keys.value.findIndex(k => k.id === key.id)
  if (idx >= 0) keys.value[idx] = key
  else keys.value.push(key)
  try {
    const b = await allocationKeysApi.balance(key.id)
    keyBalance.value[key.id] = { shares_sum: b.data.shares_sum, shares_total: b.data.shares_total, delta: b.data.delta }
  } catch { /* non-blocking */ }
}

function handleKeyDeleted(id: string) {
  keys.value = keys.value.filter(k => k.id !== id)
}

// ── Load ──────────────────────────────────────────────────────────

async function loadKeys() {
  const res = await allocationKeysApi.list()
  keys.value = res.data
  // EP-001-S07: fetch balance for each key (non-blocking, best-effort)
  await Promise.all(res.data.map(async (k) => {
    try {
      const b = await allocationKeysApi.balance(k.id)
      keyBalance.value[k.id] = { shares_sum: b.data.shares_sum, shares_total: b.data.shares_total, delta: b.data.delta }
    } catch { /* non-blocking */ }
  }))
}

const { loading, error, reload } = usePageLoader(loadKeys)
onMounted(reload)
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          <span>{{ t('nav.groups.residence') }}</span>
          <CIconChevronRight />
          <span>{{ t('nav.residence-keys') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="activeTab === 'keys'"
          size="sm"
          :icon="CIconPlus"
          :title="t('settings.keys.newKey')"
          @click="openCreateModal"
        />
        <CBtn
          v-if="activeTab === 'edd' && !eddTabRef?.showForm"
          size="sm"
          :icon="CIconPlus"
          :title="t('settings.edd.newReference')"
          @click="openEddForm"
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
          <CEyebrow>
            {{ t('page.eyebrows.allocationKeys') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('allocationKeys.title') }}
          </CHeading>
          <CMetaItem variant="sub">
            {{ t('allocationKeys.header.totalCount', { count: keys.length }) }}
          </CMetaItem>
        </header>

        <CTabs
          v-model="activeTab"
          :tabs="tabs"
        >
          <template v-if="activeTab === 'keys'">
            <template v-if="keys.length === 0">
              <CEmptyState
                :title="t('settings.keys.empty')"
                size="page"
              >
                <template #actions>
                  <CBtn
                    variant="primary"
                    :icon="CIconPlus"
                    :title="t('settings.keys.newKey')"
                    @click="openCreateModal"
                  />
                </template>
              </CEmptyState>
            </template>
            <template v-else>
              <CCard class="p-0!">
                <CDataTable
                  :columns="columns"
                  :rows="keys"
                  row-key="id"
                  :empty-text="t('settings.keys.empty')"
                >
                  <template #cell-name="{ row }">
                    <div class="flex flex-wrap items-center gap-2">
                      <strong>{{ row.name }}</strong>
                      <CBadge
                        v-if="row.key_type === 'general'"
                        variant="default"
                      >
                        {{ t('settings.keys.general') }}
                      </CBadge>
                    </div>
                  </template>
                  <template #cell-description="{ row }">
                    <span class="dim">{{ row.description || '—' }}</span>
                  </template>
                  <template #cell-shares="{ row }">
                    <span
                      v-if="keyBalance[row.id]"
                      class="data-md whitespace-nowrap"
                    >
                      {{ keyBalance[row.id].shares_sum }} / {{ keyBalance[row.id].shares_total }}
                    </span>
                    <span
                      v-else
                      class="dim"
                    >—</span>
                  </template>
                  <template #cell-balance="{ row }">
                    <CBadge
                      v-if="keyBalance[row.id] && keyBalance[row.id].delta !== 0"
                      variant="warn"
                      :title="t('settings.keys.imbalanceDetail', { delta: keyBalance[row.id].delta })"
                    >
                      {{ t('settings.keys.imbalance') }}
                    </CBadge>
                    <CBadge
                      v-else
                      variant="valid"
                    >
                      {{ t('settings.keys.balanced') }}
                    </CBadge>
                  </template>
                  <template #cell-actions="{ row }">
                    <div class="action-btns">
                      <CBtn
                        variant="icon"
                        :icon="CIconClock"
                        :title="t('settings.keys.history')"
                        :aria-label="t('settings.keys.history')"
                        @click="openAuditModal(row)"
                      />
                      <CBtn
                        variant="icon"
                        :icon="CIconEdit"
                        :title="t('common.edit')"
                        :aria-label="t('common.edit')"
                        @click="openEditModal(row)"
                      />
                      <CBtn
                        v-if="row.key_type !== 'general'"
                        variant="icon"
                        class="tone--err"
                        :icon="CIconTrash"
                        :title="t('common.delete')"
                        :aria-label="t('common.delete')"
                        @click="openDeleteKeyModal(row)"
                      />
                      <span
                        v-else
                        class="general-key-lock"
                        :title="t('settings.keys.generalProtected')"
                      >
                        <CIconLock
                          class="icon-sm"
                          aria-hidden="true"
                        />
                      </span>
                    </div>
                  </template>
                </CDataTable>
              </CCard>
            </template>
          </template>
          <template v-else>
            <EddTab ref="eddTabRef" />
          </template>
        </CTabs>
      </template>
    </div>

    <KeyFormModal
      :open="showKeyModal"
      :key-to-edit="keyToEdit"
      @update:open="showKeyModal = $event"
      @saved="handleKeySaved"
    />

    <DeleteKeyModal
      :open="showDeleteKeyModal"
      :key-to-delete="keyToDelete"
      @update:open="showDeleteKeyModal = $event"
      @deleted="handleKeyDeleted"
    />

    <KeyAuditModal
      :open="showAuditModal"
      :key-to-show="keyToShowAudit"
      @update:open="showAuditModal = $event"
    />
  </div>
</template>

<style scoped>
.dim { color: var(--color-on-surface-variant); }

.action-btns {
  display: flex;
  gap: var(--space-1);
  justify-content: flex-end;
  align-items: center;
}

.general-key-lock {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.75rem;
  height: 2.75rem;
  border-radius: var(--radius-md);
  color: var(--color-on-surface-variant);
  cursor: default;
  flex-shrink: 0;
}
</style>
