<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import { allocationKeysApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import KeyFormModal from '@/features/residence/KeyFormModal.vue'
import DeleteKeyModal from '@/features/residence/DeleteKeyModal.vue'
import type { AllocationKey } from '@/types/domain'

const { t } = useI18n()

const keys = ref<AllocationKey[]>([])
const keyBalanceDelta = ref<Record<string, number>>({})

// ── Modal state ───────────────────────────────────────────────────
const showKeyModal = ref(false)
const keyToEdit = ref<AllocationKey | null>(null)
const showDeleteKeyModal = ref(false)
const keyToDelete = ref<AllocationKey | null>(null)

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

function handleKeySaved(key: AllocationKey, balanceDelta: number | null) {
  const idx = keys.value.findIndex(k => k.id === key.id)
  if (idx >= 0) keys.value[idx] = key
  else keys.value.push(key)
  if (balanceDelta !== null) keyBalanceDelta.value[key.id] = balanceDelta
}

// ── Load ──────────────────────────────────────────────────────────

async function loadKeys() {
  const res = await allocationKeysApi.list()
  keys.value = res.data
  // EP-001-S07: fetch balance delta for each key (non-blocking, best-effort)
  await Promise.all(res.data.map(async (k) => {
    try {
      const b = await allocationKeysApi.balance(k.id)
      keyBalanceDelta.value[k.id] = b.data.delta
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
          size="sm"
          :icon="CIconPlus"
          :title="t('settings.keys.newKey')"
          @click="openCreateModal"
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
          <div class="grid grid-cols-[repeat(auto-fill,minmax(320px,1fr))] gap-4">
            <CCard
              v-for="key in keys"
              :key="key.id"
              class="key-card"
            >
              <div class="key-card__inner">
                <div class="key-info">
                  <div class="key-info__name">
                    <span class="title-sm">{{ key.name }}</span>
                    <span
                      v-if="keyBalanceDelta[key.id] !== 0 && keyBalanceDelta[key.id] !== undefined"
                      class="balance-badge"
                      :title="t('settings.keys.imbalanceDetail', { delta: keyBalanceDelta[key.id] })"
                    >
                      <CIconWarn
                        
                        class="icon-xs"
                        aria-hidden="true"
                      /> {{ t('settings.keys.imbalance') }}
                    </span>
                  </div>
                  <span
                    v-if="key.description"
                    class="key-info__desc"
                  >{{ key.description }}</span>
                </div>
                <div class="key-actions">
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :icon="CIconEdit"
                    :title="t('common.edit')"
                    @click="openEditModal(key)"
                  />
                  <CBtn
                    v-if="key.key_type !== 'general'"
                    variant="danger"
                    size="sm"
                    :icon="CIconTrash"
                    :title="t('common.delete')"
                    @click="openDeleteKeyModal(key)"
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
              </div>
            </CCard>
          </div>
        </template>
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
      @deleted="(id) => { keys.value = keys.value.filter(k => k.id !== id) }"
    />
  </div>
</template>

<style scoped>
.key-info__name { display: flex; align-items: center; gap: var(--space-2); flex-wrap: wrap; margin-bottom: 2px; }
.key-info__name .title-sm { font-weight: var(--fw-semibold); }
.key-info__desc { font-size: var(--text-xs); color: var(--color-on-surface-variant); }
.key-actions { display: flex; gap: var(--space-2); align-items: center; padding-top: var(--space-4); border-top: 1px solid var(--color-surface-container-high); margin-top: var(--space-3); }

/* keys-grid replaced by global .cards-grid */

/* Column layout: info grows, actions pinned to bottom */
.key-card :deep(.c-card) {
  height: 100%;
}

.key-card__inner {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.key-info { display: flex; flex-direction: column; gap: var(--space-1); flex: 1; }

.balance-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 2px var(--space-2);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--color-warning) 15%, transparent);
  color: var(--color-warning, #b45309);
  font-size: var(--text-2xs);
  font-weight: var(--fw-semibold);
  white-space: nowrap;
}

.general-key-lock {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-lg);
  color: var(--color-on-surface-variant);
  cursor: default;
  flex-shrink: 0;
}
.general-key-lock:hover {
  background: var(--color-surface-container-low);
  color: var(--color-on-surface);
}

.shares-section { border-top: 1px solid var(--color-outline-variant); padding-top: var(--space-4); }
.shares-loading { display: flex; align-items: center; gap: var(--space-2); color: var(--color-on-surface-variant); font-size: var(--text-sm); }

.c-chips { display: flex; flex-wrap: wrap; gap: 6px; }

.shares-summary {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--text-sm);
  color: var(--color-on-surface-variant);
}
.sum-mismatch { color: var(--color-error); }
.dim { color: var(--color-on-surface-variant); }
</style>
