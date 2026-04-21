<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CModal from '@/components/molecules/CModal.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { unitsApi, allocationKeysApi, usersApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import { useFormFeedback, SUCCESS_AUTO_CLEAR_MS } from '@/composables/useFormFeedback'
import { useConfirm } from '@/composables/useConfirm'
import type { Unit, User, UnitType, AllocationKey } from '@/types/domain'

const { t } = useI18n()
const { confirm } = useConfirm()

const activeTab = ref<'units' | 'users'>('units')
const search = ref('')

const allUnits = ref<Unit[]>([])
const allUsers = ref<User[]>([])
const shareMap = ref<Map<string, number>>(new Map())
const allKeys = ref<AllocationKey[]>([])

// Derived: the general key (key_type === 'general')
const generalKey = computed(() => allKeys.value.find(k => k.key_type === 'general') ?? null)

// ── Unit form state ───────────────────────────────────────────────
const showUnitModal = ref(false)
const unitFeedback = useFormFeedback()
const savingUnit = ref(false)
const editingUnit = ref<{
  id?: string
  unit_number: string
  unit_type: UnitType
  block: string
  staircase: string
  floor: string
  description: string
  general_shares: number
}>({
  unit_number: '', unit_type: 'apartment', block: '', staircase: '', floor: '', description: '', general_shares: 0,
})

const unitTypes: UnitType[] = ['apartment', 'cellar', 'parking', 'commercial_unit', 'other']

const units = computed(() => {
  const q = search.value.toLowerCase()
  const filtered = !q
    ? allUnits.value
    : allUnits.value.filter(u =>
      u.unit_number.toLowerCase().includes(q) ||
      (u.current_owner?.last_name ?? '').toLowerCase().includes(q)
    )
  return [...filtered].sort((a, b) =>
    a.unit_number.localeCompare(b.unit_number, undefined, { numeric: true })
  )
})

const users = computed(() => {
  const q = search.value.toLowerCase()
  if (!q) return allUsers.value
  return allUsers.value.filter(u => `${u.first_name} ${u.last_name}`.toLowerCase().includes(q))
})

async function loadData() {
  const [unitsRes, keysRes] = await Promise.all([
    unitsApi.list(),
    allocationKeysApi.list(),
  ])
  allUnits.value = unitsRes.data
  allKeys.value = keysRes.data

  const genKey = keysRes.data.find(k => k.key_type === 'general')
  if (genKey) {
    const sharesRes = await allocationKeysApi.listShares(genKey.id)
    shareMap.value = new Map(sharesRes.data.shares.map(s => [s.unit_id, s.share_value]))
  }

  try {
    const usersRes = await usersApi.list()
    allUsers.value = usersRes.data
  } catch { /* endpoint may not yet be available */ }
}

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
onMounted(reload)

function unitTypeIcon(type: string): string {
  const icons: Record<string, string> = { apartment: '🏠', cellar: '🗄️', parking: '🚗', commercial_unit: '🏪', other: '📦' }
  return icons[type] ?? '📦'
}

function unitTypeLabel(type: string): string {
  const keys: Record<string, string> = { apartment: 'apartment', cellar: 'cellar', parking: 'parking', commercial_unit: 'commercial_unit', other: 'other' }
  return keys[type] ? t(`units.types.${keys[type]}`) : type
}

function openCreateUnit() {
  editingUnit.value = { unit_number: '', unit_type: 'apartment', block: '', staircase: '', floor: '', description: '', general_shares: 0 }
  unitFeedback.reset()
  showUnitModal.value = true
}

async function openEditUnit(unit: Unit) {
  editingUnit.value = {
    id: unit.id,
    unit_number: unit.unit_number,
    unit_type: unit.unit_type,
    block: unit.block ?? '',
    staircase: unit.staircase ?? '',
    floor: unit.floor ?? '',
    description: unit.description ?? '',
    general_shares: shareMap.value.get(unit.id) ?? 0,
  }
  unitFeedback.reset()
  showUnitModal.value = true

  // If the share map doesn't have exact data yet, fetch the general key shares
  if (generalKey.value) {
    try {
      const res = await allocationKeysApi.listShares(generalKey.value.id)
      const match = res.data.shares.find((s: { unit_id: string; share_value: number }) => s.unit_id === unit.id)
      editingUnit.value.general_shares = match?.share_value ?? 0
    } catch { /* keep current value */ }
  }
}

async function saveUnit() {
  if (!editingUnit.value.unit_number.trim()) {
    unitFeedback.setError(t('common.required'))
    return
  }
  savingUnit.value = true
  unitFeedback.reset()
  try {
    const data = {
      unit_number: editingUnit.value.unit_number,
      unit_type: editingUnit.value.unit_type,
      block: editingUnit.value.block || undefined,
      staircase: editingUnit.value.staircase || undefined,
      floor: editingUnit.value.floor || undefined,
      description: editingUnit.value.description || undefined,
    }
    let unitId = editingUnit.value.id
    if (unitId) {
      await unitsApi.update(unitId, data)
    } else {
      const res = await unitsApi.create(data)
      unitId = res.data.id
    }

    // Assign general shares to the general key only
    if (generalKey.value && unitId) {
      await allocationKeysApi.setShares(generalKey.value.id, [
        { unit_id: unitId, share_value: Number(editingUnit.value.general_shares) },
      ])
    }

    unitFeedback.setSuccess(t('common.saved'))
    await loadData()
    setTimeout(() => { showUnitModal.value = false }, SUCCESS_AUTO_CLEAR_MS)
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    const msg = (e as { response?: { data?: { message?: string } } })?.response?.data?.message ?? ''
    if (status === 409 && msg === 'UNIT_NUMBER_ARCHIVED') {
      unitFeedback.setError(t('residents.unitNumberArchived'))
    } else if (status === 409) {
      unitFeedback.setError(t('residents.unitNumberExists'))
    } else {
      unitFeedback.setError(t('common.error'))
    }
  } finally {
    savingUnit.value = false
  }
}

async function archiveUnit(id: string) {
  const unit = allUnits.value.find(u => u.id === id)
  const ok = await confirm({
    title: t('residents.confirmArchiveTitle'),
    message: t('residents.confirmArchiveMessage', { number: unit?.unit_number ?? id }),
    confirmLabel: t('common.archiveLabel'),
    variant: 'danger',
  })
  if (!ok) return
  try {
    await unitsApi.archive(id)
    await loadData()
  } catch {
    unitFeedback.setError(t('common.error'))
  }
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
          {{ t('nav.residents') }}
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('residents.title') }}</span>
        </div>
      </template>
      <template #right>
        <div style="display:flex;gap:8px">
          <CBtn
            variant="primary"
            size="sm"
            @click="openCreateUnit"
          >
            <CIconPlus style="width:16px;height:16px" />
            <span class="btn-label">{{ t('residents.createUnit') }}</span>
          </CBtn>
          <CBtn
            variant="secondary"
            size="sm"
          >
            <CIconUserPlus style="width:16px;height:16px" />
            <span class="btn-label">{{ t('residents.invite') }}</span>
          </CBtn>
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
      <template v-else>
        <div class="view-header">
          <div>
            <h1 class="display-md">
              {{ t('residents.title') }}
            </h1>
            <p
              class="body-md"
              style="color:var(--color-on-surface-variant)"
            >
              {{ t('residents.summary', { units: allUnits.length, users: allUsers.length }) }}
            </p>
          </div>
        </div>

        <div class="tabs">
          <button
            v-for="tab in [
              { key: 'units', label: t('residents.tabs.units', { count: allUnits.length }) },
              { key: 'users', label: t('residents.tabs.users', { count: allUsers.length }) },
            ]"
            :key="tab.key"
            :class="['tab', { 'tab--active': activeTab === tab.key }]"
            @click="activeTab = tab.key as typeof activeTab"
          >
            {{ tab.label }}
          </button>
        </div>

        <div class="search-bar">
          <CIconSearch class="search-icon" />
          <input
            v-model="search"
            type="search"
            :placeholder="activeTab === 'units' ? t('residents.searchUnit') : t('residents.searchUser')"
            class="search-input"
          />
        </div>

        <!-- Lots -->
        <template v-if="activeTab === 'units'">
          <CCard padding="none">
            <table class="data-table">
              <thead>
                <tr>
                  <th>{{ t('common.lot') }}</th>
                  <th>{{ t('common.type') }}</th>
                  <th>{{ t('residents.location') }}</th>
                  <th>{{ t('units.shares') }}</th>
                  <th>{{ t('residents.currentOwner') }}</th>
                  <th>{{ t('common.status') }}</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="unit in units"
                  :key="unit.id"
                >
                  <td><strong class="data-md">{{ unit.unit_number }}</strong></td>
                  <td>{{ unitTypeIcon(unit.unit_type) }} {{ unitTypeLabel(unit.unit_type) }}</td>
                  <td class="dim">
                    {{ formatLocation(unit) }}
                  </td>
                  <td class="dim cell-numeric">
                    {{ shareMap.get(unit.id) ?? '—' }}
                  </td>
                  <td>
                    <span v-if="unit.current_owner">{{ unit.current_owner.first_name }} {{ unit.current_owner.last_name
                    }}</span>
                    <span
                      v-else
                      class="dim"
                    >{{ t('residents.vacant') }}</span>
                  </td>
                  <td>
                    <CBadge :variant="unit.active ? 'open' : 'closed'">
                      {{ unit.active ? t('common.active') : t('common.inactive') }}
                    </CBadge>
                  </td>
                  <td>
                    <div class="action-btns">
                      <button
                        class="icon-btn icon-btn--subtle"
                        :title="t('units.charges.title')"
                        :aria-label="t('units.charges.title')"
                        @click="$router.push(`/residents/${unit.id}/charges`)"
                      >
                        <CIconCurrency
                          width="16"
                          height="16"
                        />
                      </button>
                      <button
                        class="icon-btn"
                        :title="t('common.edit')"
                        :aria-label="t('common.edit')"
                        @click="openEditUnit(unit)"
                      >
                        <CIconEdit
                          width="16"
                          height="16"
                        />
                      </button>
                      <button
                        v-if="unit.active"
                        class="icon-btn icon-btn--danger"
                        :title="t('residents.archive')"
                        :aria-label="t('residents.archive')"
                        @click="archiveUnit(unit.id)"
                      >
                        <CIconArchive
                          width="16"
                          height="16"
                        />
                      </button>
                    </div>
                  </td>
                </tr>
                <tr v-if="units.length === 0">
                  <td
                    colspan="7"
                    style="text-align:center;color:var(--color-on-surface-variant);padding:24px"
                  >
                    {{ t('residents.unitNotFound') }}
                  </td>
                </tr>
              </tbody>
            </table>
          </CCard>
        </template>

        <!-- Utilisateurs -->
        <template v-if="activeTab === 'users'">
          <CCard padding="none">
            <table class="data-table">
              <thead>
                <tr>
                  <th>{{ t('common.name') }}</th>
                  <th>{{ t('common.email') }}</th>
                  <th>{{ t('common.roles') }}</th>
                  <th>{{ t('common.status') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="user in users"
                  :key="user.id"
                >
                  <td>
                    <div class="user-name">
                      <div class="avatar">
                        {{ user.first_name[0] }}{{ user.last_name[0] }}
                      </div>
                      <span>{{ user.first_name }} {{ user.last_name }}</span>
                    </div>
                  </td>
                  <td class="dim">
                    {{ user.email }}
                  </td>
                  <td>
                    <div class="roles">
                      <CBadge
                        v-for="role in user.roles"
                        :key="role"
                        :variant="role"
                      >
                        {{ t(`roles.${role}`) }}
                      </CBadge>
                    </div>
                  </td>
                  <td>
                    <CBadge :variant="user.active ? 'open' : 'closed'">
                      {{ user.active ? t('common.active') : t('residents.deactivated') }}
                    </CBadge>
                  </td>
                </tr>
                <tr v-if="users.length === 0">
                  <td
                    colspan="4"
                    style="text-align:center;color:var(--color-on-surface-variant);padding:24px"
                  >
                    {{ t('residents.userNotFound') }}
                  </td>
                </tr>
              </tbody>
            </table>
          </CCard>
        </template>
      </template>
      <CModal
        :open="showUnitModal"
        :title="editingUnit.id ? t('residents.editUnit') : t('residents.createUnit')"
        @update:open="showUnitModal = $event"
      >
        <div class="unit-form">
          <div class="form-grid">
            <div class="field">
              <label class="field-label">{{ t('units.number') }} *</label>
              <input
                v-model="editingUnit.unit_number"
                class="field-input"
                placeholder="A01"
              />
            </div>
            <div class="field">
              <label class="field-label">{{ t('common.type') }}</label>
              <select
                v-model="editingUnit.unit_type"
                class="field-input"
              >
                <option
                  v-for="ut in unitTypes"
                  :key="ut"
                  :value="ut"
                >
                  {{ t(`units.types.${ut}`) }}
                </option>
              </select>
            </div>
            <div class="field">
              <label class="field-label">{{ t('residents.block') }}</label>
              <input
                v-model="editingUnit.block"
                class="field-input"
              />
            </div>
            <div class="field">
              <label class="field-label">{{ t('residents.staircaseLabel') }}</label>
              <input
                v-model="editingUnit.staircase"
                class="field-input"
              />
            </div>
            <div class="field">
              <label class="field-label">{{ t('residents.floorLabel') }}</label>
              <input
                v-model="editingUnit.floor"
                class="field-input"
              />
            </div>
            <div class="field field--full">
              <label class="field-label">{{ t('residents.description') }}</label>
              <textarea
                v-model="editingUnit.description"
                class="field-input"
              ></textarea>
            </div>
          </div>

          <!-- Tantièmes généraux (clé Charges Générales) -->
          <div class="shares-section">
            <h3 class="shares-title">
              {{ t('units.generalShares') }}
            </h3>
            <div class="field">
              <label class="field-label">{{ generalKey ? generalKey.name : t('units.shares') }}</label>
              <input
                v-model.number="editingUnit.general_shares"
                type="number"
                step="1"
                min="0"
                class="field-input"
                style="max-width:160px"
              />
              <p class="field-hint">
                {{ t('units.generalSharesHint') }}
              </p>
            </div>
          </div>
        </div>
        <template #footer>
          <CFormFeedback
            :status="unitFeedback.status.value"
            :message="unitFeedback.message.value"
            style="flex:1"
          />
          <CBtn
            variant="ghost"
            @click="showUnitModal = false"
          >
            {{ t('common.cancel') }}
          </CBtn>
          <CBtn
            variant="primary"
            :loading="savingUnit"
            :disabled="savingUnit"
            @click="saveUnit"
          >
            {{ savingUnit ? t('common.saving') : t('common.save') }}
          </CBtn>
        </template>
      </CModal>
    </div>
  </div>
</template>

<style scoped>
.dim {
  color: var(--color-on-surface-variant);
}

.user-name {
  display: flex;
  align-items: center;
  gap: 10px;
}

.avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--gradient-primary);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: var(--text-xs);
  font-weight: var(--fw-bold);
  flex-shrink: 0;
}

.roles {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
}

.action-btns {
  display: flex;
  gap: 4px;
  justify-content: flex-end;
  align-items: center;
}

.unit-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.shares-section {
  border-top: 1px solid var(--color-outline-variant);
  padding-top: var(--space-4);
}

.shares-title {
  font-size: var(--text-md);
  font-weight: var(--fw-semibold);
  margin-bottom: var(--space-3);
}

.shares-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-3);
}
</style>
