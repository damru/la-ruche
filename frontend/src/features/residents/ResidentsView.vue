<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconCurrency from '@/components/atoms/CIconCurrency.svg'
import CIconUserPlus from '@/components/atoms/CIconUserPlus.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconArchive from '@/components/atoms/CIconArchive.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconRefresh from '@/components/atoms/CIconRefresh.svg'
import CIconArrowRight from '@/components/atoms/CIconArrowRight.svg'
import CIconShield from '@/components/atoms/CIconShield.svg'
import InviteUserModal from './InviteUserModal.vue'
import CreateResidentModal from './CreateResidentModal.vue'
import SendInviteModal from './SendInviteModal.vue'
import EditUnitModal from './EditUnitModal.vue'
import ArchiveUnitModal from './ArchiveUnitModal.vue'
import RevokeInviteModal from './RevokeInviteModal.vue'
import EditResidentModal from './EditResidentModal.vue'
import ArchiveUserModal from './ArchiveUserModal.vue'
import EditRolesModal from './EditRolesModal.vue'
import TransferOwnershipModal from './TransferOwnershipModal.vue'
import { unitsApi, allocationKeysApi, usersApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import type { Unit, UnitType, User, AllocationKey } from '@/types/domain'
import { UNIT_TYPES } from '@/utils/constants'

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
const unitToEdit = ref<Unit | null>(null)

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

const UNIT_TYPE_ICONS: Record<UnitType, string> = {
  apartment: '🏠',
  cellar: '🗄️',
  parking: '🚗',
  commercial_unit: '🏪',
  other: '📦',
}

function unitTypeIcon(type: string): string {
  return UNIT_TYPE_ICONS[type as UnitType] ?? UNIT_TYPE_ICONS[UNIT_TYPES[UNIT_TYPES.length - 1]]
}

function unitTypeLabel(type: string): string {
  return UNIT_TYPES.includes(type as UnitType) ? t(`units.types.${type}`) : type
}

function openCreateUnit() {
  unitToEdit.value = null
  showUnitModal.value = true
}

function openEditUnit(unit: Unit) {
  unitToEdit.value = unit
  showUnitModal.value = true
}

// ── Archive unit modal ────────────────────────────────────────────

const showArchiveModal = ref(false)
const unitToArchive = ref<Unit | null>(null)

function openArchiveModal(unit: Unit) {
  unitToArchive.value = unit
  showArchiveModal.value = true
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
  { key: 'status', label: t('status') },
  { key: 'actions', label: '', width: '1%' },
])

const userColumns = computed(() => [
  { key: 'name', label: t('lastName') },
  { key: 'email', label: t('common.email') },
  { key: 'phone', label: t('common.phone') },
  { key: 'roles', label: t('common.roles') },
  { key: 'status', label: t('status') },
  { key: 'actions', label: '', width: '1%' },
])

// ── Invite user modal (email-first flow) ─────────────────────────
const showInviteModal = ref(false)

// ── Create resident modal ─────────────────────────────────────────
const showCreateResidentModal = ref(false)

// ── Send invite to no_access user ─────────────────────────────────
const showSendInviteModal = ref(false)
const userToInvite = ref<User | null>(null)

function openSendInviteModal(user: User) {
  userToInvite.value = user
  showSendInviteModal.value = true
}

// ── Revoke invite ─────────────────────────────────────────────────
const showRevokeModal = ref(false)
const userToRevoke = ref<User | null>(null)

function openRevokeModal(user: User) {
  userToRevoke.value = user
  showRevokeModal.value = true
}

// ── Edit resident modal ───────────────────────────────────────────
const showEditResidentModal = ref(false)
const userToEdit = ref<User | null>(null)

function openEditResidentModal(user: User) {
  userToEdit.value = user
  showEditResidentModal.value = true
}

// ── Archive user modal ────────────────────────────────────────────
const showArchiveUserModal = ref(false)
const userToArchive = ref<User | null>(null)

function openArchiveUserModal(user: User) {
  userToArchive.value = user
  showArchiveUserModal.value = true
}

// ── Reactivate user (inline, no modal needed) ─────────────────────
async function reactivateUser(user: User) {
  await usersApi.reactivate(user.id)
  await loadData()
}

// ── Edit roles modal ──────────────────────────────────────────────
const showEditRolesModal = ref(false)
const userForRoles = ref<User | null>(null)

function openEditRolesModal(user: User) {
  userForRoles.value = user
  showEditRolesModal.value = true
}

// ── Transfer ownership modal ──────────────────────────────────────
const showTransferModal = ref(false)
const unitToTransfer = ref<Unit | null>(null)

function openTransferModal(unit: Unit) {
  unitToTransfer.value = unit
  showTransferModal.value = true
}</script>

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
        <div class="flex gap-2">
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
            :title="t('residents.addResident')"
            @click="showCreateResidentModal = true"
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
                  {{ row.active ? t('active') : t('common.inactive') }}
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
                    :aria-label="t('residents.transferOwnership')"
                    :title="t('residents.transferOwnership')"
                    :icon="CIconArrowRight"
                    @click="openTransferModal(row)"
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
                    :title="t('common.archiveLabel')"
                    :aria-label="t('common.archiveLabel')"
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
                <span
                  v-if="row.email"
                  class="dim"
                >{{ row.email }}</span>
                <span
                  v-else
                  class="dim"
                >—</span>
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
                  v-else-if="row.status === 'no_access'"
                  variant="disabled"
                >
                  {{ t('residents.noAccess') }}
                </CBadge>
                <CBadge
                  v-else-if="row.status === 'archived'"
                  variant="closed"
                >
                  {{ t('residents.archived') }}
                </CBadge>
                <CBadge
                  v-else
                  variant="open"
                >
                  {{ t('active') }}
                </CBadge>
              </template>
              <template #cell-actions="{ row }">
                <div class="action-btns">
                  <CBtn
                    v-if="row.status === 'no_access'"
                    variant="icon"
                    :title="t('residents.sendInvite')"
                    :aria-label="t('residents.sendInvite')"
                    :icon="CIconUserPlus"
                    @click="openSendInviteModal(row)"
                  />
                  <CBtn
                    v-if="row.status === 'pending_invite'"
                    variant="icon"
                    :title="t('residents.confirmRevokeTitle')"
                    :aria-label="t('residents.confirmRevokeTitle')"
                    :icon="CIconTrash"
                    class="tone--err"
                    @click="openRevokeModal(row)"
                  />
                  <template v-if="row.status !== 'pending_invite'">
                    <CBtn
                      variant="icon"
                      :title="t('residents.editResident')"
                      :aria-label="t('residents.editResident')"
                      :icon="CIconEdit"
                      @click="openEditResidentModal(row)"
                    />
                    <CBtn
                      variant="icon"
                      :title="t('residents.editRoles')"
                      :aria-label="t('residents.editRoles')"
                      :icon="CIconShield"
                      @click="openEditRolesModal(row)"
                    />
                    <CBtn
                      v-if="row.status !== 'archived'"
                      variant="icon"
                      :title="t('residents.archiveUser')"
                      :aria-label="t('residents.archiveUser')"
                      :icon="CIconArchive"
                      class="tone--err"
                      @click="openArchiveUserModal(row)"
                    />
                    <CBtn
                      v-if="row.status === 'archived'"
                      variant="icon"
                      :title="t('residents.reactivateUser')"
                      :aria-label="t('residents.reactivateUser')"
                      :icon="CIconRefresh"
                      @click="reactivateUser(row)"
                    />
                  </template>
                </div>
              </template>
            </CDataTable>
          </CCard>
        </template>
      </template>
      <EditUnitModal
        :open="showUnitModal"
        :units="allUnits"
        :unit="unitToEdit"
        :general-key-id="generalKey?.id ?? null"
        :general-key-name="generalKey?.name ?? null"
        :share-value="unitToEdit ? shareMap.get(unitToEdit.id) : undefined"
        @update:open="showUnitModal = $event"
        @saved="loadData"
      />

      <!-- Archive unit modal -->
      <ArchiveUnitModal
        :open="showArchiveModal"
        :unit="unitToArchive"
        @update:open="showArchiveModal = $event"
        @archived="loadData"
      />

      <!-- Invite user modal (email-first flow) -->
      <InviteUserModal
        :open="showInviteModal"
        :units="allUnits"
        @update:open="showInviteModal = $event"
        @invited="loadData"
      />

      <!-- Create resident modal (name-first, no invite required) -->
      <CreateResidentModal
        :open="showCreateResidentModal"
        :units="allUnits"
        @update:open="showCreateResidentModal = $event"
        @created="loadData"
      />

      <!-- Send invite to no_access user -->
      <SendInviteModal
        :open="showSendInviteModal"
        :user="userToInvite"
        @update:open="showSendInviteModal = $event"
        @invited="loadData"
      />

      <!-- Revoke invite modal -->
      <RevokeInviteModal
        :open="showRevokeModal"
        :user="userToRevoke"
        @update:open="showRevokeModal = $event"
        @revoked="loadData"
      />

      <!-- Edit resident modal -->
      <EditResidentModal
        :open="showEditResidentModal"
        :user="userToEdit"
        @update:open="showEditResidentModal = $event"
        @saved="loadData"
      />

      <!-- Archive user modal -->
      <ArchiveUserModal
        :open="showArchiveUserModal"
        :user="userToArchive"
        @update:open="showArchiveUserModal = $event"
        @archived="loadData"
      />

      <!-- Edit roles modal -->
      <EditRolesModal
        :open="showEditRolesModal"
        :user="userForRoles"
        @update:open="showEditRolesModal = $event"
        @saved="loadData"
      />

      <!-- Transfer ownership modal -->
      <TransferOwnershipModal
        :open="showTransferModal"
        :unit="unitToTransfer"
        :residents="allUsers"
        @update:open="showTransferModal = $event"
        @transferred="loadData"
      />
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
  gap: var(--space-1);
  justify-content: flex-end;
  align-items: center;
}


</style>
