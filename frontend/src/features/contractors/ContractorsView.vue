<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useAuthStore } from '@/stores/auth'
import CBadge from '@/components/atoms/CBadge.vue'
import CIconTile from '@/components/atoms/CIconTile.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
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
  const [buildingRes, contractorsRes] = await Promise.all([
    buildingApi.get(),
    contractorsApi.list(),
  ])
  buildingName.value = buildingRes.data.name
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

// ── Domain filter toggle ──────────────────────────────────────────
function toggleDomain(domain: string) {
  const idx = selectedDomains.value.indexOf(domain)
  if (idx >= 0) selectedDomains.value.splice(idx, 1)
  else selectedDomains.value.push(domain)
}

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
    feedback.setError(toErrorMessage(err))
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
    deleteFeedback.setError(toErrorMessage(err))
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
          <CIconChevronRight class="icon-xs" />
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
          <CIconPlus class="icon-sm" />
          <span class="btn-label">{{ t('contractors.newContractor') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="flex flex-col p-10 max-w-[1200px] gap-6 max-sm:p-5">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else>
        <header class="flex flex-col items-start gap-2">
          <CEyebrow>{{ t('page.eyebrows.contractors') }}</CEyebrow>
          <CHeading type="header">
            {{ t('contractors.title') }}
          </CHeading>
          <p class="body-md text-on-surface-variant">
            {{ t('contractors.summary', { count: allContractors.length }) }}
          </p>
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

        <!-- Domain filter chips -->
        <div
          class="domain-filter"
          role="group"
          :aria-label="t('contractors.domainsLabel')"
        >
          <button
            v-for="opt in domainOptions"
            :key="opt.value"
            :class="['domain-chip', { 'domain-chip--active': selectedDomains.includes(opt.value) }]"
            :aria-pressed="selectedDomains.includes(opt.value)"
            @click="toggleDomain(opt.value)"
          >
            {{ opt.label }}
          </button>
        </div>

        <!-- Empty state -->
        <div
          v-if="filtered.length === 0"
          class="py-8 text-center text-md text-on-surface-variant"
        >
          <CIconUsers class="empty-state__icon" />
          <p class="body-md">
            {{ search || selectedDomains.length ? t('contractors.noResults') : t('contractors.noContractors') }}
          </p>
          <CBtn
            v-if="canEdit && !search && !selectedDomains.length"
            variant="primary"
            size="sm"
            @click="openCreate"
          >
            <CIconPlus class="icon-sm" />
            {{ t('contractors.newContractor') }}
          </CBtn>
        </div>

        <!-- ── List view ─────────────────────────────────────────────── -->
        <div
          v-else-if="viewMode === 'list'"
          class="contractor-list"
          role="list"
        >
          <div
            v-for="c in filtered"
            :key="c.id"
            class="contractor-row"
            role="listitem"
            tabindex="-1"
            @click="openContractorDetail(c.id)"
          >
            <CIconTile :class="['contractor-row__icon', domainTone(c.domains)]">
              <component
                :is="domainIcon(c.domains)"
                aria-hidden="true"
              />
            </CIconTile>
            <button
              class="contractor-row__name"
              @click.stop="openContractorDetail(c.id)"
            >
              {{ c.name }}
            </button>
            <div class="contractor-row__domains text-xs">
              <CMetaLine>
                <CBadge
                  v-for="d in c.domains.slice(0, 2)"
                  :key="d"
                  variant="default"
                >{{ t(`contractors.domains.${d}`, d) }}</CBadge>
                <CBadge
                  v-if="c.domains.length > 2"
                  variant="default"
                >+{{ c.domains.length - 2 }}</CBadge>
              </CMetaLine>
            </div>
            <div class="contractor-row__contacts">
              <a
                v-if="c.email"
                :href="`mailto:${c.email}`"
                class="contractor-row__contact"
                @click.stop
              >{{ c.email }}</a>
              <a
                v-if="c.phone"
                :href="`tel:${c.phone}`"
                class="contractor-row__contact"
                @click.stop
              >{{ c.phone }}</a>
            </div>
            <div
              v-if="canEdit"
              class="contractor-row__actions"
            >
              <CBtn
                variant="ghost"
                size="sm"
                :aria-label="`${t('common.edit')} ${c.name}`"
                @click.stop="openEdit(c)"
              >
                <CIconEdit class="icon-sm" />
              </CBtn>
              <CBtn
                variant="ghost"
                size="sm"
                :aria-label="`${t('common.delete')} ${c.name}`"
                @click.stop="openDeleteModal(c)"
              >
                <CIconTrash class="icon-sm" />
              </CBtn>
            </div>
          </div>
        </div>

        <!-- ── Grid view ─────────────────────────────────────────────── -->
        <div
          v-else
          class="contractors-grid"
        >
          <CCard
            v-for="c in filtered"
            :key="c.id"
            interactive
            class="activity-card"
            tabindex="-1"
            @click="openContractorDetail(c.id)"
          >
            <div class="activity-card__header-row activity-card__header-row--with-icon">
              <CIconTile :class="['activity-card__icon', domainTone(c.domains)]">
                <component
                  :is="domainIcon(c.domains)"
                  aria-hidden="true"
                />
              </CIconTile>
              <button
                class="activity-card__title-btn"
                @click.stop="openContractorDetail(c.id)"
              >
                {{ c.name }}
              </button>
            </div>
            <div class="activity-card__chips">
              <CBadge
                v-for="d in c.domains.slice(0, 2)"
                :key="d"
                variant="default"
              >{{ t(`contractors.domains.${d}`, d) }}</CBadge>
              <CBadge
                v-if="c.domains.length > 2"
                variant="default"
              >+{{ c.domains.length - 2 }}</CBadge>
            </div>
            <div class="activity-card__stack">
              <p
                v-if="c.notes"
                class="body-xs"
              >
                {{ c.notes }}
              </p>
              <div
                v-if="c.email || c.phone"
                class="activity-card__contacts"
              >
                <a
                  v-if="c.email"
                  :href="`mailto:${c.email}`"
                  class="activity-card__contact-link"
                  @click.stop
                >{{ c.email }}</a>
                <a
                  v-if="c.phone"
                  :href="`tel:${c.phone}`"
                  class="activity-card__contact-link"
                  @click.stop
                >{{ c.phone }}</a>
              </div>
              <div
                v-if="canEdit"
                class="activity-card__actions"
              >
                <CBtn
                  variant="ghost"
                  size="sm"
                  :aria-label="`${t('common.edit')} ${c.name}`"
                  @click.stop="openEdit(c)"
                >
                  <CIconEdit class="icon-sm" />
                  {{ t('common.edit') }}
                </CBtn>
                <CBtn
                  variant="ghost"
                  size="sm"
                  :aria-label="`${t('common.delete')} ${c.name}`"
                  @click.stop="openDeleteModal(c)"
                >
                  <CIconTrash class="icon-sm" />
                  {{ t('common.delete') }}
                </CBtn>
              </div>
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
        class="flex flex-col gap-4"
        @submit.prevent="save"
      >
        <CFormInput
          v-model="form.name"
          :label="t('contractors.name')"
          :placeholder="t('contractors.namePlaceholder')"
          required
        />
        <CFormInput
          v-model="form.email"
          :label="t('contractors.email')"
          type="email"
          :placeholder="t('contractors.emailPlaceholder')"
        />
        <CFormInput
          v-model="form.phone"
          :label="t('contractors.phone')"
          type="tel"
          :placeholder="t('contractors.phonePlaceholder')"
        />
        <CFormChipSelect
          v-model="form.domains"
          :label="t('contractors.domainsLabel')"
          :options="domainOptions"
          class="flex flex-col gap-1 col-span-full"
        />
        <CFormTextarea
          v-model="form.notes"
          :label="t('contractors.notes')"
          :rows="3"
          :placeholder="t('contractors.notesPlaceholder')"
          class="flex flex-col gap-1 col-span-full"
        />
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
/* ── Empty state ─────────────────────────────────────────────────── */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-12) 0;
  color: var(--color-on-surface-variant);
}

.empty-state__icon {
  width: 48px;
  height: 48px;
  opacity: 0.4;
}

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
  border-radius: var(--radius-md);
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

/* ── Domain filter chips ─────────────────────────────────────────── */
.domain-filter {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.domain-chip {
  display: inline-flex;
  align-items: center;
  height: 28px;
  padding: 0 var(--space-3);
  border-radius: var(--radius-full);
  border: 1px solid var(--color-outline-variant);
  background: var(--color-surface);
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: background 150ms ease-out, border-color 150ms ease-out, color 150ms ease-out;
}

.domain-chip:hover {
  background: var(--color-secondary-container);
  color: var(--color-on-surface);
}

.domain-chip--active {
  background: var(--color-primary-container);
  border-color: var(--color-primary);
  color: var(--color-on-primary);
}

/* ── List view ───────────────────────────────────────────────────── */
.contractor-list {
  background: var(--color-surface-container-lowest);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.contractor-row {
  display: grid;
  grid-template-columns: 36px minmax(120px, 1fr) auto minmax(0, 210px) auto;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-outline-variant);
  cursor: pointer;
  transition: background 150ms ease-out;
}

.contractor-row:last-child {
  border-bottom: none;
}

.contractor-row:hover {
  background: var(--color-surface-container-low);
}

.contractor-row__name {
  background: none;
  border: none;
  padding: 0;
  font: inherit;
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
  color: var(--color-on-surface);
  text-align: left;
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
}

.contractor-row__name:hover,
.contractor-row__name:focus-visible {
  color: var(--color-primary);
  outline: none;
}

.contractor-row__contacts {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.contractor-row__contact {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
  text-decoration: none;
  overflow: hidden;
  text-overflow: ellipsis;
}

.contractor-row__contact:hover {
  color: var(--color-primary);
  text-decoration: underline;
}

.contractor-row__actions {
  display: flex;
  gap: var(--space-1);
  flex-shrink: 0;
}

/* ── Grid view ───────────────────────────────────────────────────── */
.contractors-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--space-4);
}

.activity-card__title-btn {
  background: none;
  border: none;
  padding: 0;
  font: inherit;
  font-size: var(--text-md, 1rem);
  font-weight: var(--fw-semibold);
  color: var(--color-on-surface);
  text-align: left;
  cursor: pointer;
  line-height: 1.3;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.activity-card__title-btn:hover,
.activity-card__title-btn:focus-visible {
  color: var(--color-primary);
  outline: none;
}

.activity-card__contacts {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.activity-card__contact-link {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
  text-decoration: none;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.activity-card__contact-link:hover {
  color: var(--color-primary);
  text-decoration: underline;
}

/* ── Responsive ──────────────────────────────────────────────────── */
@media (max-width: 640px) {
  .contractor-row {
    grid-template-columns: 36px minmax(0, 1fr) auto;
  }

  .contractor-row__domains,
  .contractor-row__contacts {
    display: none;
  }
}

/* ── Icon size utilities ─────────────────────────────────────────── */
.icon-xs { width: var(--icon-xs); height: var(--icon-xs); flex-shrink: 0; }
.icon-sm { width: var(--icon-sm); height: var(--icon-sm); flex-shrink: 0; }

.activity-card { display: flex; flex-direction: column; gap: var(--space-3); }
.activity-card__header-row { display: flex; align-items: center; gap: var(--space-3); min-width: 0; }
.activity-card__header-row--with-icon { align-items: center; }
.activity-card__icon { flex-shrink: 0; align-self: flex-start; }
.activity-card__stack { display: flex; flex-direction: column; gap: var(--space-3); }
.activity-card__chips { flex: 1; align-items: center; font-size: var(--text-xs); flex-wrap: wrap-reverse; display: flex; flex-wrap: wrap; gap: var(--space-1) var(--space-2); }
.activity-card__chips :deep(svg), .activity-card__chips > * > svg { width: var(--icon-xs); height: var(--icon-xs); }
.activity-card__contacts { display: flex; flex-direction: column; gap: var(--space-1); }
.activity-card__actions { display: flex; flex-wrap: wrap; gap: var(--space-1) var(--space-2); }
.activity-card__contact-link { font-size: var(--text-xs); color: var(--color-primary); text-decoration: none; }
.activity-card__contact-link:hover { text-decoration: underline; }
@media (max-width: 720px) { .activity-card__header-row { align-items: flex-start; } .activity-card__header-row--with-icon { align-items: center; } }
</style>
