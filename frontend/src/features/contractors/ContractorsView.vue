<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useAuthStore } from '@/stores/auth'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
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
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import { buildingApi, contractorsApi } from '@/api'
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
const authStore = useAuthStore()

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const allContractors = ref<Contractor[]>([])
const search = ref('')

// ── Permissions ────────────────────────────────────────────────────
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
      c.domains.some(d => d.toLowerCase().includes(q)),
  )
})

// ── Sorted domains list (alphabetical by translated label) ────────
const DOMAINS = computed(() =>
  [...DOMAINS_RAW].sort((a, b) =>
    t(`contractors.domains.${a}`).localeCompare(t(`contractors.domains.${b}`)),
  ),
)

// ── Form modal ────────────────────────────────────────────────────
const showModal = ref(false)
const editingContractor = ref<Contractor | null>(null)
const saving = ref(false)
const feedback = useFormFeedback()
const { schedule: scheduleModalClose, forceExpire: forceModalClose } = useAutoClose()
watch(showModal, (open) => { if (!open) forceModalClose() })

const defaultForm = () => ({
  name: '',
  email: '',
  phone: '',
  domains: [] as string[],
  notes: '',
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
    domains: [...c.domains],
    notes: c.notes ?? '',
  }
  feedback.reset()
  showModal.value = true
}

function closeModal() {
  showModal.value = false
}

async function save() {
  if (feedback.status.value === 'success') { forceModalClose(); return }
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
      domains: form.value.domains,
      notes: form.value.notes.trim() || undefined,
    }
    if (editingContractor.value) {
      await contractorsApi.update(editingContractor.value.id, payload)
    } else {
      await contractorsApi.create(payload)
    }
    feedback.setSuccess(t('common.saved'))
    scheduleModalClose(() => { showModal.value = false; reload() })
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
const { schedule: scheduleDeleteClose, forceExpire: forceDeleteClose } = useAutoClose()
watch(showDeleteModal, (open) => { if (!open) forceDeleteClose() })

function openDeleteModal(c: Contractor) {
  contractorToDelete.value = c
  deleteFeedback.reset()
  showDeleteModal.value = true
}

async function confirmDelete() {
  if (deleteFeedback.status.value === 'success') { forceDeleteClose(); return }
  if (!contractorToDelete.value) return
  deletingId.value = contractorToDelete.value.id
  deleteFeedback.reset()
  try {
    await contractorsApi.delete(contractorToDelete.value.id)
    deleteFeedback.setSuccess(t('contractors.deleteSuccess'))
    scheduleDeleteClose(() => { showDeleteModal.value = false; reload() })
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

        <!-- Contractors grid -->
        <div
          v-else
          class="contractors-grid"
        >
          <CCard
            v-for="c in filtered"
            :key="c.id"
            class="contractor-card"
          >
            <div class="activity-card__head">
              <div class="activity-card__header-row activity-card__header-row--with-icon">
                <div :class="['activity-card__icon', 'contractor-card__icon', domainTone(c.domains)]">
                  <component
                    :is="domainIcon(c.domains)"
                    class="contractor-card__type-icon"
                    aria-hidden="true"
                  />
                </div>
                <h3 class="activity-card__title contractor-name">
                  {{ c.name }}
                </h3>
              </div>

              <div class="activity-card__chips">
                <CBadge
                  v-for="d in c.domains"
                  :key="d"
                  variant="default"
                >
                  {{ t(`contractors.domains.${d}`, d) }}
                </CBadge>
              </div>
            </div>

            <div
              v-if="c.email || c.phone"
              class="contractor-contacts"
            >
              <a
                v-if="c.email"
                :href="`mailto:${c.email}`"
                class="contact-link"
              >{{ c.email }}</a>
              <a
                v-if="c.phone"
                :href="`tel:${c.phone}`"
                class="contact-link"
              >{{ c.phone }}</a>
            </div>

            <p
              v-if="c.notes"
              class="contractor-notes body-xs"
            >
              {{ c.notes }}
            </p>

            <div
              v-if="canEdit"
              class="contractor-actions"
            >
              <CBtn
                variant="ghost"
                size="sm"
                :aria-label="t('common.edit') + ' ' + c.name"
                @click="openEdit(c)"
              >
                <CIconEdit />
                {{ t('common.edit') }}
              </CBtn>
              <CBtn
                variant="danger"
                size="sm"
                :aria-label="t('common.delete') + ' ' + c.name"
                @click="openDeleteModal(c)"
              >
                <CIconTrash />
                {{ t('common.delete') }}
              </CBtn>
            </div>
          </CCard>
        </div>
      </template>
    </div>

    <!-- Create / Edit modal -->
    <CModal
      :open="showModal"
      :title="editingContractor ? t('contractors.editContractor') : t('contractors.newContractor')"
      @update:open="showModal = $event"
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
        <!-- Domains: multi-select checkboxes, sorted alphabetically -->
        <div class="form-field form-field--full">
          <label class="form-label">{{ t('contractors.domainsLabel') }}</label>
          <div class="domains-checkboxes">
            <label
              v-for="d in DOMAINS"
              :key="d"
              class="checkbox-label"
            >
              <input
                v-model="form.domains"
                type="checkbox"
                :value="d"
              />
              {{ t(`contractors.domains.${d}`) }}
            </label>
          </div>
        </div>
        <!-- Notes textarea -->
        <div class="form-field form-field--full">
          <label class="form-label">{{ t('contractors.notes') }}</label>
          <textarea
            v-model="form.notes"
            class="form-input"
            rows="3"
            :placeholder="t('contractors.notesPlaceholder')"
          ></textarea>
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
          :closing="feedback.status.value === 'success'"
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
      @update:open="showDeleteModal = $event"
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
          :closing="deleteFeedback.status.value === 'success'"
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

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-12) 0;
  color: var(--color-on-surface-variant);
}

/* Grid */
.contractors-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--space-4);
}

/* Card */
.contractor-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.contractor-name {
  margin: 0;
}

.contractor-card__icon {
  background: var(--tone-bg, var(--color-secondary-container));
  color: var(--tone-fg, var(--color-primary));
}

.contractor-card__type-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.contractor-contacts {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.contact-link {
  font-size: var(--text-xs);
  color: var(--color-primary);
  text-decoration: none;
}

.contact-link:hover {
  text-decoration: underline;
}

.contractor-notes {
  color: var(--color-on-surface-variant);
  margin: 0;
  white-space: pre-wrap;
}

.contractor-actions {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
  margin-top: auto;
  padding-top: var(--space-3);
  border-top: 1px solid var(--color-surface-container-high);
}

/* Domains checkbox grid */
.domains-checkboxes {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2) var(--space-4);
  padding: var(--space-2) 0;
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
  font-weight: var(--fw-medium);
}
</style>
