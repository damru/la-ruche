<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { allocationKeysApi, unitsApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { AllocationKey } from '@/types/domain'

const { t } = useI18n()

const keys = ref<AllocationKey[]>([])
const keyBalanceDelta = ref<Record<string, number>>({})  // EP-001-S07: imbalance badge per key
const keyFeedback = useFormFeedback()

// ── Modal state ───────────────────────────────────────────────────
const showKeyModal = ref(false)
const { schedule: scheduleKeyClose, forceExpire: forceKeyClose } = useAutoClose()
watch(showKeyModal, (open) => { if (!open) forceKeyClose() })
const modalForm = ref<{
  id?: string
  name: string
  description: string
  shares: { unit_id: string; unit_number: string; share_value: number }[]
  sharesTotal: number
}>({ name: '', description: '', shares: [], sharesTotal: 0 })
const modalLoading = ref(false)
const modalSaving = ref(false)

// ── Derived ───────────────────────────────────────────────────────

const modalSharesSum = computed(() =>
  modalForm.value.shares.reduce((acc, s) => acc + (s.share_value || 0), 0)
)

function sortSharesByUnit(shares: { unit_id: string; unit_number: string; share_value: number }[]) {
  return [...shares].sort((a, b) =>
    a.unit_number.localeCompare(b.unit_number, undefined, { numeric: true })
  )
}

const isSumMismatch = computed(() => {
  const total = modalForm.value.sharesTotal
  return total > 0 && modalSharesSum.value !== total
})

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

// ── Modal open ────────────────────────────────────────────────────

async function openCreateModal() {
  keyFeedback.reset()
  modalForm.value = { name: '', description: '', shares: [], sharesTotal: 0 }
  showKeyModal.value = true
  modalLoading.value = true
  try {
    // Load all units to allow share assignment at creation time
    const res = await unitsApi.list()
    modalForm.value.shares = sortSharesByUnit(
      res.data
        .filter(u => u.active)
        .map(u => ({ unit_id: u.id, unit_number: u.unit_number, share_value: 0 }))
    )
  } catch { /* units failed to load — shares section will show empty state */ }
  finally { modalLoading.value = false }
}

async function openEditModal(key: AllocationKey) {
  keyFeedback.reset()
  modalForm.value = { id: key.id, name: key.name, description: key.description ?? '', shares: [], sharesTotal: 0 }
  showKeyModal.value = true
  modalLoading.value = true
  try {
    const res = await allocationKeysApi.listShares(key.id)
    modalForm.value.shares = sortSharesByUnit(res.data.shares.map(s => ({ ...s })))
    modalForm.value.sharesTotal = res.data.shares_total
  } catch { /* non-blocking — shares section will show empty state */ }
  finally { modalLoading.value = false }
}

// ── Save ──────────────────────────────────────────────────────────

async function saveKey() {
  if (keyFeedback.status.value === 'success') { forceKeyClose(); return }
  if (!modalForm.value.name.trim()) {
    keyFeedback.setError(t('common.required'))
    return
  }
  keyFeedback.reset()
  modalSaving.value = true
  try {
    let keyId = modalForm.value.id
    if (keyId) {
      const res = await allocationKeysApi.update(keyId, {
        name: modalForm.value.name,
        description: modalForm.value.description || undefined,
      })
      const idx = keys.value.findIndex(k => k.id === keyId)
      if (idx >= 0) keys.value[idx] = res.data
    } else {
      const res = await allocationKeysApi.create({
        name: modalForm.value.name,
        description: modalForm.value.description || undefined,
      })
      keyId = res.data.id
      keys.value.push(res.data)
    }

    if (modalForm.value.shares.length > 0) {
      await allocationKeysApi.setShares(keyId!, modalForm.value.shares.map(s => ({
        unit_id: s.unit_id,
        share_value: Number(s.share_value),
      })))
    }

    // EP-001-S07: refresh balance delta after save
    try {
      const bal = await allocationKeysApi.balance(keyId!)
      keyBalanceDelta.value[keyId!] = bal.data.delta
    } catch { /* non-blocking */ }

    keyFeedback.setSuccess(t('common.saved'))
    scheduleKeyClose(() => { showKeyModal.value = false })
  } catch (e: unknown) {
    keyFeedback.setError((e as { response?: { data?: { message?: string } } })?.response?.data?.message ?? t('common.error'))
  } finally {
    modalSaving.value = false
  }
}

// ── Delete ────────────────────────────────────────────────────────

const showDeleteKeyModal = ref(false)
const keyToDelete = ref<AllocationKey | null>(null)
const deletingKeyInProgress = ref(false)
const deleteKeyFeedback = useFormFeedback()
const { schedule: scheduleDeleteKeyClose, forceExpire: forceDeleteKeyClose } = useAutoClose()
watch(showDeleteKeyModal, (open) => { if (!open) forceDeleteKeyClose() })

function openDeleteKeyModal(key: AllocationKey) {
  keyToDelete.value = key
  deleteKeyFeedback.reset()
  showDeleteKeyModal.value = true
}

async function confirmDeleteKey() {
  if (deleteKeyFeedback.status.value === 'success') { forceDeleteKeyClose(); return }
  if (!keyToDelete.value) return
  const keyId = keyToDelete.value.id
  deletingKeyInProgress.value = true
  deleteKeyFeedback.reset()
  try {
    await allocationKeysApi.delete(keyId)
    keys.value = keys.value.filter(k => k.id !== keyId)
    deleteKeyFeedback.setSuccess(t('settings.keys.deleteSuccess'))
    scheduleDeleteKeyClose(() => { showDeleteKeyModal.value = false })
  } catch (e: unknown) {
    deleteKeyFeedback.setError((e as { response?: { data?: { message?: string } } })?.response?.data?.message ?? t('common.error'))
  } finally {
    deletingKeyInProgress.value = false
  }
}

const { loading, error, reload } = usePageLoader(loadKeys, t('common.loadError'))
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
          variant="secondary"
          size="sm"
          @click="openCreateModal"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('settings.keys.newKey') }}</span>
        </CBtn>
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
          <h1 class="display-md">
            {{ t('nav.residence-keys') }}
          </h1>
        </header>

        <p
          v-if="keys.length === 0"
          class="empty-state"
        >
          {{ t('settings.keys.empty') }}
        </p>

        <div class="cards-grid">
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
                      width="14"
                      height="14"
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
                  @click="openEditModal(key)"
                >
                  <CIconEdit />
                  {{ t('common.edit') }}
                </CBtn>
                <CBtn
                  v-if="key.key_type !== 'general'"
                  variant="danger"
                  size="sm"
                  @click="openDeleteKeyModal(key)"
                >
                  <CIconTrash />
                  {{ t('common.delete') }}
                </CBtn>
                <span
                  v-else
                  class="general-key-lock"
                  :title="t('settings.keys.generalProtected')"
                >
                  <CIconLock
                    width="16"
                    height="16"
                    aria-hidden="true"
                  />
                </span>
              </div>
            </div>
          </CCard>
        </div>
      </template>
    </div>

    <!-- Create / Edit modal -->
    <CModal
      :open="showKeyModal"
      size="lg"
      :closable="!modalSaving"
      :title="modalForm.id ? t('settings.keys.editKey') : t('settings.keys.newKey')"
      @update:open="showKeyModal = $event"
    >
      <div class="form-grid">
        <div class="form-grid">
          <CFormInput
            v-model="modalForm.name"
            :label="t('common.name')"
            :placeholder="t('settings.keys.namePlaceholder')"
            required
            class="field--full"
          />
          <CFormTextarea
            v-model="modalForm.description"
            :label="t('settings.keys.descriptionLabel')"
            :placeholder="t('settings.keys.descriptionPlaceholder')"
            class="field--full"
          />
        </div>

        <div class="shares-section">
          <h3 class="shares-title">
            {{ t('settings.keys.showShares') }}
          </h3>

          <div
            v-if="modalLoading"
            class="shares-loading"
          >
            <CIconSpinner
              class="spin"
              width="18"
              height="18"
            />
            <span>{{ t('common.loading') }}</span>
          </div>
          <template v-else>
            <p
              v-if="modalForm.shares.length === 0"
              class="dim"
            >
              {{ t('settings.keys.noUnits') }}
            </p>
            <template v-else>
              <div class="c-chips">
                <CFormChipInput
                  v-for="share in modalForm.shares"
                  :key="share.unit_id"
                  v-model="share.share_value"
                  :label="share.unit_number"
                  type="number"
                  :step="1"
                  :min="0"
                />
              </div>
              <div class="shares-summary">
                <span>{{ t('common.total') }} :</span>
                <span
                  :class="{ 'sum-mismatch': isSumMismatch }"
                  style="font-weight:var(--fw-semibold)"
                >
                  {{ modalSharesSum }}
                </span>
                <span
                  v-if="modalForm.sharesTotal > 0"
                  class="dim"
                >/ {{ modalForm.sharesTotal }}</span>
              </div>
              <CAlert
                v-if="isSumMismatch"
                variant="warning"
                style="margin-top:var(--space-3)"
              >
                {{ t('settings.keys.sumMismatch') }}
              </CAlert>
            </template>
          </template>
        </div>
      </div>

      <template #footer>
        <CFormFeedback
          :status="keyFeedback.status.value"
          :message="keyFeedback.message.value"
          style="flex:1"
        />
        <CBtn
          variant="ghost"
          @click="showKeyModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="modalSaving"
          :closing="keyFeedback.status.value === 'success'"
          :disabled="modalSaving || modalLoading"
          @click="saveKey"
        >
          {{ modalSaving ? t('common.saving') : t('common.save') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Delete key modal -->
    <CModal
      :open="showDeleteKeyModal"
      :title="t('settings.keys.confirmDeleteTitle')"
      size="sm"
      @update:open="showDeleteKeyModal = $event"
    >
      <p class="body-md">
        {{ t('settings.keys.confirmDeleteMessage', { name: keyToDelete?.name ?? '' }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="deleteKeyFeedback.status.value"
          :message="deleteKeyFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showDeleteKeyModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="deletingKeyInProgress"
          :closing="deleteKeyFeedback.status.value === 'success'"
          @click="confirmDeleteKey"
        >
          {{ t('common.deleteLabel') }}
        </CBtn>
      </template>
    </CModal>
  </div>
</template>

<style scoped>
.key-info__name { display: flex; align-items: center; gap: var(--space-2); flex-wrap: wrap; margin-bottom: 2px; }
.key-info__name .title-sm { font-weight: var(--fw-semibold); }
.key-info__desc { font-size: var(--text-xs); color: var(--color-on-surface-variant); }
.key-actions { display: flex; gap: var(--space-2); align-items: center; padding-top: var(--space-4); border-top: 1px solid var(--color-surface-container-high); margin-top: var(--space-3); }

.empty-state { text-align: center; color: var(--color-on-surface-variant); padding: 24px 0; }

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
  border-radius: var(--radius-sm);
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
  border-radius: var(--radius-sm);
  color: var(--color-on-surface-variant);
  cursor: default;
  flex-shrink: 0;
}
.general-key-lock:hover {
  background: var(--color-surface-container-low);
  color: var(--color-on-surface);
}

.shares-section { border-top: 1px solid var(--color-outline-variant); padding-top: var(--space-4); }
.shares-title { font-size: var(--text-md); font-weight: var(--fw-semibold); margin-bottom: var(--space-3); }

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
