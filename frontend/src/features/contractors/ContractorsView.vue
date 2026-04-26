<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAuthStore } from '@/stores/auth'
import CAlert from '@/components/atoms/CAlert.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconSearch from '@/components/atoms/CIconSearch.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconUsers from '@/components/atoms/CIconUsers.svg'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import { buildingApi, contractorsApi } from '@/api'
import type { Contractor } from '@/types/domain'

const { t } = useI18n()
const authStore = useAuthStore()

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const allContractors = ref<Contractor[]>([])
const search = ref('')

// ── Permissions ────────────────────────────────────────────────────
// Managers and board members can create/edit/delete contractors.
// All other roles can read (catalog is public to all building users).
const canEdit = computed(() =>
  authStore.hasAnyRole(['manager', 'board_member']),
)

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [buildingRes, contractorsRes] = await Promise.all([
    buildingApi.get(),
    contractorsApi.list(),
  ])
  buildingName.value = buildingRes.data.name
  allContractors.value = contractorsRes.data
}

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
onMounted(reload)

// ── Filtered list ─────────────────────────────────────────────────
const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return allContractors.value
  return allContractors.value.filter(
    c =>
      c.name.toLowerCase().includes(q) ||
      (c.email ?? '').toLowerCase().includes(q) ||
      (c.domain ?? '').toLowerCase().includes(q),
  )
})

// ── Form modal ────────────────────────────────────────────────────
const showModal = ref(false)
const editingContractor = ref<Contractor | null>(null)
const saving = ref(false)
const feedback = useFormFeedback()
const actionError = ref('')

const DOMAINS = [
  'maintenance', 'insurance', 'service', 'energy',
  'water', 'cleaning', 'green_spaces', 'other',
]

const defaultForm = () => ({
  name: '',
  email: '',
  phone: '',
  domain: '',
})
const form = ref(defaultForm())

function openCreate() {
  editingContractor.value = null
  form.value = defaultForm()
  feedback.reset()
  showModal.value = true
}

function openEdit(c: Contractor) {
  editingContractor.value = c
  form.value = {
    name: c.name,
    email: c.email ?? '',
    phone: c.phone ?? '',
    domain: c.domain ?? '',
  }
  feedback.reset()
  showModal.value = true
}

function closeModal() {
  showModal.value = false
}

async function save() {
  if (!form.value.name.trim()) {
    feedback.setError(t('contractors.nameRequired'))
    return
  }
  saving.value = true
  feedback.reset()
  try {
    const payload = {
      name: form.value.name.trim(),
      email: form.value.email.trim() || undefined,
      phone: form.value.phone.trim() || undefined,
      domain: form.value.domain || undefined,
    }
    if (editingContractor.value) {
      await contractorsApi.update(editingContractor.value.id, payload)
    } else {
      await contractorsApi.create(payload)
    }
    showModal.value = false
    await reload()
  } catch (err) {
    feedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    saving.value = false
  }
}

// ── Delete ────────────────────────────────────────────────────────
const deletingId = ref<string | null>(null)
const showDeleteModal = ref(false)
const deleteFeedback = useFormFeedback()
const contractorToDelete = ref<Contractor | null>(null)

function openDeleteModal(c: Contractor) {
  contractorToDelete.value = c
  deleteFeedback.reset()
  showDeleteModal.value = true
}

async function confirmDelete() {
  if (!contractorToDelete.value) return
  deletingId.value = contractorToDelete.value.id
  deleteFeedback.reset()
  try {
    await contractorsApi.delete(contractorToDelete.value.id)
    showDeleteModal.value = false
    await reload()
  } catch (err) {
    deleteFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    deletingId.value = null
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('contractors.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canEdit"
          variant="primary"
          size="sm"
          @click="openCreate"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('contractors.newContractor') }}</span>
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
        <!-- Header + search -->
        <div class="view-header">
          <div>
            <h1 class="display-md">
              {{ t('contractors.title') }}
            </h1>
            <p
              class="body-md"
              style="color:var(--color-on-surface-variant)"
            >
              {{ t('contractors.summary', { count: allContractors.length }) }}
            </p>
          </div>
        </div>

        <CAlert
          v-if="actionError"
          variant="error"
          :message="actionError"
          class="mb-4"
        />

        <!-- Search bar -->
        <div class="search-bar">
          <CIconSearch class="search-icon" />
          <input
            v-model="search"
            type="search"
            class="search-input"
            :placeholder="t('contractors.searchPlaceholder')"
          />
        </div>

        <!-- Empty state -->
        <div
          v-if="filtered.length === 0"
          class="empty-state"
        >
          <CIconUsers
            width="48"
            height="48"
            style="color:var(--color-on-surface-variant)"
          />
          <p class="body-md">
            {{ search ? t('contractors.noResults') : t('contractors.noContractors') }}
          </p>
        </div>

        <!-- Contractors table -->
        <CCard
          v-else
          style="min-width:0;overflow-x:auto"
        >
          <table class="data-table">
            <thead>
              <tr>
                <th>{{ t('contractors.name') }}</th>
                <th>{{ t('contractors.domain') }}</th>
                <th>{{ t('contractors.email') }}</th>
                <th>{{ t('contractors.phone') }}</th>
                <th v-if="canEdit"></th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="c in filtered"
                :key="c.id"
              >
                <td class="font-medium">
                  {{ c.name }}
                </td>
                <td>
                  <span v-if="c.domain">{{ t(`contractors.domains.${c.domain}`, c.domain) }}</span>
                  <span
                    v-else
                    style="color:var(--color-on-surface-variant)"
                  >—</span>
                </td>
                <td>
                  <a
                    v-if="c.email"
                    :href="`mailto:${c.email}`"
                    class="link"
                  >{{ c.email }}</a>
                  <span
                    v-else
                    style="color:var(--color-on-surface-variant)"
                  >—</span>
                </td>
                <td>
                  <a
                    v-if="c.phone"
                    :href="`tel:${c.phone}`"
                    class="link"
                  >{{ c.phone }}</a>
                  <span
                    v-else
                    style="color:var(--color-on-surface-variant)"
                  >—</span>
                </td>
                <td
                  v-if="canEdit"
                  class="actions-cell"
                >
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :aria-label="t('common.edit')"
                    @click="openEdit(c)"
                  >
                    <CIconEdit
                      width="16"
                      height="16"
                    />
                  </CBtn>
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :aria-label="t('common.delete')"
                    @click="openDeleteModal(c)"
                  >
                    <CIconTrash
                      width="16"
                      height="16"
                    />
                  </CBtn>
                </td>
              </tr>
            </tbody>
          </table>
        </CCard>
      </template>
    </div>

    <!-- Create / Edit modal -->
    <CModal
      :open="showModal"
      :title="editingContractor ? t('contractors.editContractor') : t('contractors.newContractor')"
      @close="closeModal"
    >
      <form
        class="form-grid"
        @submit.prevent="save"
      >
        <div class="form-field">
          <label class="form-label">{{ t('contractors.name') }} *</label>
          <input
            v-model="form.name"
            class="form-input"
            :placeholder="t('contractors.namePlaceholder')"
            required
          />
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('contractors.domain') }}</label>
          <select
            v-model="form.domain"
            class="form-input"
          >
            <option value="">
              —
            </option>
            <option
              v-for="d in DOMAINS"
              :key="d"
              :value="d"
            >
              {{ t(`contractors.domains.${d}`) }}
            </option>
          </select>
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('contractors.email') }}</label>
          <input
            v-model="form.email"
            type="email"
            class="form-input"
            :placeholder="t('contractors.emailPlaceholder')"
          />
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('contractors.phone') }}</label>
          <input
            v-model="form.phone"
            type="tel"
            class="form-input"
            :placeholder="t('contractors.phonePlaceholder')"
          />
        </div>
      </form>
      <template #footer>
        <CFormFeedback
          :status="feedback.status.value"
          :message="feedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="closeModal"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="saving"
          @click="save"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Delete confirmation modal -->
    <CModal
      :open="showDeleteModal"
      :title="t('contractors.deleteTitle')"
      @close="showDeleteModal = false"
    >
      <p class="body-md">
        {{ t('contractors.deleteConfirm', { name: contractorToDelete?.name }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="deleteFeedback.status.value"
          :message="deleteFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showDeleteModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="!!deletingId"
          @click="confirmDelete"
        >
          {{ t('common.delete') }}
        </CBtn>
      </template>
    </CModal>
  </div>
</template>

<style scoped>
.view-page {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.view-container {
  flex: 1;
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  overflow-y: auto;
}

.view-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.search-bar {
  position: relative;
  max-width: 360px;
}

.search-icon {
  position: absolute;
  left: var(--space-3);
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: var(--color-on-surface-variant);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: var(--space-2) var(--space-3) var(--space-2) calc(var(--space-3) + 20px);
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-on-surface);
  font-size: var(--text-sm);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-12) 0;
  color: var(--color-on-surface-variant);
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--text-sm);
}

.data-table th,
.data-table td {
  padding: var(--space-3) var(--space-4);
  text-align: left;
  border-bottom: 1px solid var(--color-outline-variant);
}

.data-table th {
  font-weight: var(--font-medium);
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.data-table tr:last-child td {
  border-bottom: none;
}

.font-medium {
  font-weight: var(--font-medium);
}

.link {
  color: var(--color-primary);
  text-decoration: none;
}

.link:hover {
  text-decoration: underline;
}

.actions-cell {
  display: flex;
  gap: var(--space-1);
  justify-content: flex-end;
}

.form-grid {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.form-label {
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  color: var(--color-on-surface);
}

.form-input {
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-on-surface);
  font-size: var(--text-sm);
  width: 100%;
}

.form-input:focus {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-color: transparent;
}

.topbar-breadcrumb {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-sm);
  color: var(--color-on-surface-variant);
}

.topbar-breadcrumb span:last-child {
  color: var(--color-on-surface);
  font-weight: var(--font-medium);
}

.mb-4 {
  margin-bottom: var(--space-4);
}
</style>
