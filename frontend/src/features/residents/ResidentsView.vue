<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CFormRow from '@/components/molecules/CFormRow.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconCurrency from '@/components/atoms/CIconCurrency.svg'
import CIconUserPlus from '@/components/atoms/CIconUserPlus.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconArchive from '@/components/atoms/CIconArchive.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import InviteUserModal from './InviteUserModal.vue'
import { unitsApi, allocationKeysApi, usersApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { Unit, User, UnitType, AllocationKey } from '@/types/domain'

const { t } = useI18n()

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
const { schedule: scheduleUnitClose, forceExpire: forceUnitClose } = useAutoClose()
watch(showUnitModal, (open) => { if (!open) forceUnitClose() })
const editingUnit = ref<{
  id?: string
  unit_number: string
  unit_type: UnitType
  block: string
  staircase: string
  floor: string
  description: string
  general_shares: number | string
}>({
  unit_number: '', unit_type: 'apartment', block: '', staircase: '', floor: '', description: '', general_shares: 0,
})

const unitTypes: UnitType[] = ['apartment', 'cellar', 'parking', 'commercial_unit', 'other']

const unitTypeOptions = computed(() =>
  unitTypes.map(ut => ({ value: ut, label: t(`units.types.${ut}`) })),
)

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

const { loading, error, reload } = usePageLoader(loadData)
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
  if (unitFeedback.status.value === 'success') { forceUnitClose(); return }
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
    scheduleUnitClose(() => { showUnitModal.value = false })
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

// ── Archive unit modal ────────────────────────────────────────────

const showArchiveModal = ref(false)
const unitToArchive = ref<Unit | null>(null)
const archivingUnit = ref(false)
const archiveFeedback = useFormFeedback()
const { schedule: scheduleArchiveClose, forceExpire: forceArchiveClose } = useAutoClose()
watch(showArchiveModal, (open) => { if (!open) forceArchiveClose() })

function openArchiveModal(unit: Unit) {
  unitToArchive.value = unit
  archiveFeedback.reset()
  showArchiveModal.value = true
}

async function confirmArchiveUnit() {
  if (archiveFeedback.status.value === 'success') { forceArchiveClose(); return }
  if (!unitToArchive.value) return
  archivingUnit.value = true
  archiveFeedback.reset()
  try {
    await unitsApi.archive(unitToArchive.value.id)
    archiveFeedback.setSuccess(t('residents.archiveSuccess'))
    await loadData()
    scheduleArchiveClose(() => { showArchiveModal.value = false })
  } catch {
    archiveFeedback.setError(t('common.error'))
  } finally {
    archivingUnit.value = false
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

const unitColumns = computed(() => [
  { key: 'unit_number', label: t('common.lot') },
  { key: 'unit_type', label: t('common.type') },
  { key: 'location', label: t('residents.location') },
  { key: 'shares', label: t('units.shares'), numeric: true },
  { key: 'owner', label: t('residents.currentOwner') },
  { key: 'status', label: t('common.status') },
  { key: 'actions', label: '', width: '1%' },
])

const userColumns = computed(() => [
  { key: 'name', label: t('common.name') },
  { key: 'email', label: t('common.email') },
  { key: 'phone', label: t('common.phone') },
  { key: 'roles', label: t('common.roles') },
  { key: 'status', label: t('common.status') },
  { key: 'actions', label: '', width: '1%' },
])

// ── Invite user modal ─────────────────────────────────────────────
const showInviteModal = ref(false)

// ── Revoke invite ─────────────────────────────────────────────────
const showRevokeModal = ref(false)
const userToRevoke = ref<User | null>(null)
const revokingInvite = ref(false)
const revokeFeedback = useFormFeedback()
const { schedule: scheduleRevokeClose, forceExpire: forceRevokeClose } = useAutoClose()
watch(showRevokeModal, (open) => { if (!open) forceRevokeClose() })

function openRevokeModal(user: User) {
  userToRevoke.value = user
  revokeFeedback.reset()
  showRevokeModal.value = true
}

async function confirmRevokeInvite() {
  if (revokeFeedback.status.value === 'success') { forceRevokeClose(); return }
  if (!userToRevoke.value?.invite_token_id) return
  revokingInvite.value = true
  revokeFeedback.reset()
  try {
    await usersApi.revokeInvite(userToRevoke.value.invite_token_id)
    revokeFeedback.setSuccess(t('residents.revokeSuccess'))
    await loadData()
    scheduleRevokeClose(() => { showRevokeModal.value = false })
  } catch {
    revokeFeedback.setError(t('common.error'))
  } finally {
    revokingInvite.value = false
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ t('nav.residents') }}
          <CIconChevronRight />
          <span>{{ t('residents.title') }}</span>
        </div>
      </template>
      <template #right>
        <div style="display:flex;gap:8px">
          <CBtn
            variant="primary"
            size="sm"
            shrink="sm"
            :icon="CIconPlus"
            :title="t('residents.createUnit')"
            @click="openCreateUnit"
          />
          <CBtn
            variant="secondary"
            size="sm"
            shrink="sm"
            :icon="CIconUserPlus"
            :title="t('residents.invite')"
            @click="showInviteModal = true"
          />
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
        <header class="view-header">
          <CEyebrow>
            {{ t('page.eyebrows.residents') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('residents.title') }}
          </CHeading>
          <CMetaLine>
            <CMetaItem variant="sub">
              {{ t('residents.header.unitsCount', { count: allUnits.length }) }}
            </CMetaItem>
            <CSeparator />
            <CMetaItem variant="sub">
              {{ t('residents.header.usersCount', { count: allUsers.length }) }}
            </CMetaItem>
          </CMetaLine>
        </header>

        <CTabs
          v-model="activeTab"
          variant="filter"
          :tabs="[
            { value: 'units', label: t('residents.tabs.units', { count: allUnits.length }) },
            { value: 'users', label: t('residents.tabs.users', { count: allUsers.length }) },
          ]"
        >
          <CFormInput
            v-model="search"
            type="search"
            :placeholder="activeTab === 'units' ? t('residents.searchUnit') : t('residents.searchUser')"
          >
            <template #prefix>
              <CIconSearch />
            </template>
          </CFormInput>
        </CTabs>
  
        <template v-if="activeTab === 'units'">
          <CCard class="p-0!">
            <CDataTable
              :columns="unitColumns"
              :rows="units"
              row-key="id"
              :empty-text="t('residents.unitNotFound')"
            >
              <template #cell-unit_number="{ row }">
                <strong class="data-md">{{ row.unit_number }}</strong>
              </template>
              <template #cell-unit_type="{ row }">
                {{ unitTypeIcon(row.unit_type) }} {{ unitTypeLabel(row.unit_type) }}
              </template>
              <template #cell-location="{ row }">
                <span class="dim">{{ formatLocation(row) }}</span>
              </template>
              <template #cell-shares="{ row }">
                <span class="dim">{{ shareMap.get(row.id) ?? '—' }}</span>
              </template>
              <template #cell-owner="{ row }">
                <span v-if="row.current_owner">{{ row.current_owner.first_name }} {{ row.current_owner.last_name }}</span>
                <span
                  v-else
                  class="dim"
                >{{ t('residents.vacant') }}</span>
              </template>
              <template #cell-status="{ row }">
                <CBadge :variant="row.active ? 'open' : 'closed'">
                  {{ row.active ? t('common.active') : t('common.inactive') }}
                </CBadge>
              </template>
              <template #cell-actions="{ row }">
                <div class="action-btns">
                  <CBtn
                    variant="icon"
                    :aria-label="t('units.charges.title')"
                    :title="t('units.charges.title')"
                    :icon="CIconCurrency"
                    @click="$router.push(`/residents/${row.id}/charges`)"
                  />
                  <CBtn
                    variant="icon"
                    :aria-label="t('common.edit')"
                    :title="t('common.edit')"
                    :icon="CIconEdit"
                    @click="openEditUnit(row)"
                  />
                  <CBtn
                    v-if="row.active"
                    variant="icon"
                    :title="t('residents.archive')"
                    :aria-label="t('residents.archive')"
                    :icon="CIconArchive"
                    class="tone--err"
                    @click="openArchiveModal(row)"
                  />
                </div>
              </template>
            </CDataTable>
          </CCard>
        </template>

        <!-- Utilisateurs -->
        <template v-if="activeTab === 'users'">
          <CCard class="p-0!">
            <CDataTable
              :columns="userColumns"
              :rows="users"
              row-key="id"
              :empty-text="t('residents.userNotFound')"
            >
              <template #cell-name="{ row }">
                <div class="user-name">
                  <CAvatar
                    v-if="row.status !== 'pending_invite'"
                    :name="`${row.first_name} ${row.last_name}`"
                  />
                  <span v-if="row.status !== 'pending_invite'">{{ row.first_name }} {{ row.last_name }}</span>
                  <span
                    v-else
                    class="dim"
                  >{{ t('residents.pendingInvite') }}</span>
                </div>
              </template>
              <template #cell-email="{ row }">
                <span class="dim">{{ row.email }}</span>
              </template>
              <template #cell-phone="{ row }">
                <a
                  v-if="row.phone"
                  :href="`tel:${row.phone}`"
                  class="phone-link"
                >{{ row.phone }}</a>
                <span v-else>—</span>
              </template>
              <template #cell-roles="{ row }">
                <div class="roles">
                  <CBadge
                    v-for="role in row.roles"
                    :key="role"
                    :variant="role"
                  >
                    {{ t(`roles.${role}`) }}
                  </CBadge>
                </div>
              </template>
              <template #cell-status="{ row }">
                <CBadge
                  v-if="row.status === 'pending_invite'"
                  variant="pending"
                >
                  {{ t('residents.pendingInvite') }}
                </CBadge>
                <CBadge
                  v-else
                  :variant="row.active ? 'open' : 'closed'"
                >
                  {{ row.active ? t('common.active') : t('residents.deactivated') }}
                </CBadge>
              </template>
              <template #cell-actions="{ row }">
                <div class="action-btns">
                  <CBtn
                    v-if="row.status === 'pending_invite'"
                    variant="icon"
                    :title="t('residents.revokeInvite')"
                    :aria-label="t('residents.revokeInvite')"
                    :icon="CIconTrash"
                    class="tone--err"
                    @click="openRevokeModal(row)"
                  />
                </div>
              </template>
            </CDataTable>
          </CCard>
        </template>
      </template>
      <CModal
        :open="showUnitModal"
        :title="editingUnit.id ? t('residents.editUnit') : t('residents.createUnit')"
        :closable="!savingUnit"
        @update:open="showUnitModal = $event"
      >
        <CForm
          id="unit-form"
          @submit="saveUnit"
        >
          <CFormRow :cols="2">
            <CFormInput
              v-model="editingUnit.unit_number"
              :label="t('units.number')"
              placeholder="A01"
              required
            />
            <CFormSelect
              v-model="editingUnit.unit_type"
              :label="t('common.type')"
              :options="unitTypeOptions"
            />
            <CFormInput
              v-model="editingUnit.block"
              :label="t('residents.block')"
            />
            <CFormInput
              v-model="editingUnit.staircase"
              :label="t('residents.staircaseLabel')"
            />
            <CFormInput
              v-model="editingUnit.floor"
              :label="t('residents.floorLabel')"
            />
            <CFormTextarea
              v-model="editingUnit.description"
              :label="t('residents.description')"
              class="flex flex-col gap-1 col-span-full"
            />
          </CFormRow>

          <!-- Tantièmes généraux (clé Charges Générales) -->
          <div class="shares-section">
            <h3 class="text-md font-semibold mb-3">
              {{ t('units.generalShares') }}
            </h3>
            <CFormInput
              v-model="editingUnit.general_shares"
              :label="generalKey ? generalKey.name : t('units.shares')"
              type="number"
              :step="1"
              :min="0"
              :hint="t('units.generalSharesHint')"
              style="max-width:240px"
            />
          </div>
        </CForm>
        <template #footer>
          <CFormActions
            :status="unitFeedback.status.value"
            :message="unitFeedback.message.value"
          >
            <CBtn
              variant="ghost"
              :title="t('common.cancel')"
              @click="showUnitModal = false"
            />
            <CBtn
              variant="primary"
              type="submit"
              form="unit-form"
              :title="t('common.save')"
              :loading="savingUnit"
              :closing="unitFeedback.status.value === 'success'"
              :disabled="savingUnit"
            />
          </CFormActions>
        </template>
      </CModal>

      <!-- Archive unit modal -->
      <CModal
        :open="showArchiveModal"
        :title="t('residents.confirmArchiveTitle')"
        size="sm"
        @update:open="showArchiveModal = $event"
      >
        <p class="body-md">
          {{ t('residents.confirmArchiveMessage', { number: unitToArchive?.unit_number ?? '' }) }}
        </p>
        <template #footer>
          <CFormActions
            :status="archiveFeedback.status.value"
            :message="archiveFeedback.message.value"
          >
            <CBtn
              variant="ghost"
              :title="t('common.cancel')"
              @click="showArchiveModal = false"
            />
            <CBtn
              variant="danger"
              :title="t('common.archiveLabel')"
              :loading="archivingUnit"
              :closing="archiveFeedback.status.value === 'success'"
              @click="confirmArchiveUnit"
            />
          </CFormActions>
        </template>
      </CModal>

      <!-- Invite user modal -->
      <InviteUserModal
        :open="showInviteModal"
        :units="allUnits"
        @update:open="showInviteModal = $event"
        @invited="loadData"
      />

      <!-- Revoke invite modal -->
      <CModal
        :open="showRevokeModal"
        :title="t('residents.confirmRevokeTitle')"
        size="sm"
        @update:open="showRevokeModal = $event"
      >
        <p class="body-md">
          {{ t('residents.confirmRevokeMessage', { email: userToRevoke?.email ?? '' }) }}
        </p>
        <template #footer>
          <CFormActions
            :status="revokeFeedback.status.value"
            :message="revokeFeedback.message.value"
          >
            <CBtn
              variant="ghost"
              :title="t('common.cancel')"
              @click="showRevokeModal = false"
            />
            <CBtn
              variant="danger"
              :title="t('residents.revokeInvite')"
              :loading="revokingInvite"
              :closing="revokeFeedback.status.value === 'success'"
              @click="confirmRevokeInvite"
            />
          </CFormActions>
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

.roles {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
}

.phone-link {
  color: var(--color-primary);
  text-decoration: none;
}
.phone-link:hover { text-decoration: underline; }

.action-btns {
  display: flex;
  gap: 4px;
  justify-content: flex-end;
  align-items: center;
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

</style>
