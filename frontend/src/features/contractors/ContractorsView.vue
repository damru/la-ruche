<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useAuthStore } from '@/stores/auth'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CMetaLine from '@/components/molecules/CMetaLine.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/organisms/CPageError.vue'
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
const router = useRouter()
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

function openContractorDetail(contractorId: string) {
  void router.push({ name: 'contractor-detail', params: { id: contractorId } })
}

// ── Sorted domains list (alphabetical by translated label) ────────
const domainOptions = computed(() =>
  [...DOMAINS_RAW]
    .map(value => ({ value, label: t(`contractors.domains.${value}`) }))
    .sort((a, b) => a.label.localeCompare(b.label)),
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
          <CIconChevronRight />
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
        <header class="view-header">
          <CEyebrow>
            {{ t('page.eyebrows.contractors') }}
          </CEyebrow>
          <h1 class="display-md">
            {{ t('contractors.title') }}
          </h1>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('contractors.summary', { count: allContractors.length }) }}
          </p>
        </header>

        <!-- Search bar -->
        <CFormInput
          v-model="search"
          type="search"
          :placeholder="t('contractors.searchPlaceholder')"
        >
          <template #prefix>
            <CIconSearch />
          </template>
        </CFormInput>

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
            interactive
            class="activity-card"
            tabindex="0"
            @click="openContractorDetail(c.id)"
            @keydown.enter.prevent="openContractorDetail(c.id)"
            @keydown.space.prevent="openContractorDetail(c.id)"
          >
            <div class="activity-card__header-row activity-card__header-row--with-icon">
              <div :class="['activity-card__icon', 'contractor-card__icon', domainTone(c.domains)]">
                <component
                  :is="domainIcon(c.domains)"
                  aria-hidden="true"
                />
              </div>
              <h3 class="activity-card__title">
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
            <div class="activity-card__stack">
              <p
                v-if="c.notes"
                class="body-xs"
              >
                {{ c.notes }}
              </p>

              <CMetaLine
                v-if="c.email || c.phone"
                class="activity-card__contacts"
              >
                <a
                  v-if="c.email"
                  :href="`mailto:${c.email}`"
                  class="activity-card__line activity-card__contact-link"
                  @click.stop
                ><span aria-hidden="true">✉️</span>{{ c.email }}</a>
                <a
                  v-if="c.phone"
                  :href="`tel:${c.phone}`"
                  class="activity-card__line activity-card__contact-link"
                  @click.stop
                ><span aria-hidden="true">📞</span>{{ c.phone }}</a>
              </CMetaLine>

              <div
                v-if="canEdit"
                class="activity-card__actions"
              >
                <CBtn
                  variant="ghost"
                  size="sm"
                  :aria-label="t('common.edit') + ' ' + c.name"
                  @click.stop="openEdit(c)"
                >
                  <CIconEdit />
                  {{ t('common.edit') }}
                </CBtn>
                <CBtn
                  variant="danger"
                  size="sm"
                  :aria-label="t('common.delete') + ' ' + c.name"
                  @click.stop="openDeleteModal(c)"
                >
                  <CIconTrash />
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
        class="form-grid"
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
          class="form-field--full"
        />
        <CFormTextarea
          v-model="form.notes"
          :label="t('contractors.notes')"
          :rows="3"
          :placeholder="t('contractors.notesPlaceholder')"
          class="form-field--full"
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

</style>
