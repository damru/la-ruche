<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
import { useRole } from '@/composables/useRole'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CLinkedDocumentsModal from '@/components/organisms/CLinkedDocumentsModal.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CCard from '@/components/molecules/CCard.vue'
import CCardActions from '@/components/molecules/CCardActions.vue'
import CCardNote from '@/components/molecules/CCardNote.vue'
import CReminderCard from '@/components/organisms/CReminderCard.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs, { type Tab } from '@/components/atoms/CTabs.vue'
import CIconCheckCircle from '@/components/atoms/CIconCheckCircle.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconHome from '@/components/atoms/CIconHome.svg'
import CIconLockOpen from '@/components/atoms/CIconLockOpen.svg'
import CIconShield from '@/components/atoms/CIconShield.svg'
import CIconUpload from '@/components/atoms/CIconUpload.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import ClaimCreateModal from './ClaimCreateModal.vue'
import ClaimEditModal from './ClaimEditModal.vue'
import ClaimTransmitModal from './ClaimTransmitModal.vue'
import ClaimCloseModal from './ClaimCloseModal.vue'
import ClaimDeleteModal from './ClaimDeleteModal.vue'
import { claimsApi, documentsApi, unitsApi, usersApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import type { Claim, ClaimStatus, ClaimType, Document, Unit, User } from '@/types/domain'

const { t } = useI18n()
const { formatCurrency, formatDate } = useFormatters()
const { hasAnyRole } = useRole()

const buildingName = ref('')
const allClaims = ref<Claim[]>([])
const allUnits = ref<Unit[]>([])
const allUsers = ref<User[]>([])
const filter = ref<'all' | ClaimStatus>('all')
const canManage = computed(() => hasAnyRole(['manager', 'board_member']))

const showCreateModal = ref(false)
const claimToEdit = ref<Claim | null>(null)
const claimToTransmit = ref<Claim | null>(null)
const claimToClose = ref<Claim | null>(null)
const claimToDelete = ref<Claim | null>(null)
const claimDocumentsTarget = ref<Claim | null>(null)
const claimDocuments = ref<Document[]>([])
const loadingClaimDocuments = ref(false)

async function loadData() {
  const [building, claimsResponse, unitsResponse, usersResponse] = await Promise.all([
    useBuildingStore().fetch(),
    claimsApi.list(),
    unitsApi.list(),
    usersApi.list(),
  ])

  buildingName.value = building.name
  allClaims.value = claimsResponse.data
  allUnits.value = unitsResponse.data
  allUsers.value = usersResponse.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

const unitsById = computed(() =>
  new Map(allUnits.value.map(unit => [unit.id, unit])),
)

const claimsById = computed(() =>
  new Map(allClaims.value.map(claim => [claim.id, claim])),
)

const claims = computed(() =>
  (filter.value === 'all'
    ? allClaims.value
    : allClaims.value.filter(claim => claim.status === filter.value))
    .slice()
    .sort((left, right) => right.updated_at.localeCompare(left.updated_at)),
)

const activeClaimsCount = computed(() =>
  allClaims.value.filter(claim => claim.status !== 'closed').length,
)

const claimCounts = computed(() =>
  allClaims.value.reduce(
    (counts, claim) => {
      counts.all += 1
      if (claim.status === 'open') counts.open += 1
      else if (claim.status === 'in_progress') counts.in_progress += 1
      else if (claim.status === 'closed') counts.closed += 1
      return counts
    },
    { all: 0, open: 0, in_progress: 0, closed: 0 },
  ),
)

const claimTabs = computed(() => [
  { key: 'all', label: t('common.all'), count: claimCounts.value.all },
  { key: 'open', label: t('claims.filters.open'), count: claimCounts.value.open },
  { key: 'in_progress', label: t('claims.filters.inProgress'), count: claimCounts.value.in_progress },
  { key: 'closed', label: t('claims.filters.closed'), count: claimCounts.value.closed },
])

const claimTabsList = computed<Tab[]>(() =>
  claimTabs.value.map(t => ({ value: t.key, label: t.label, count: t.count }))
)

const legalCardLinks = computed(() => [
  {
    label: t('claims.legalCard.currentLinkLabel'),
    href: t('claims.legalCard.currentLinkHref'),
  },
  {
    label: t('claims.legalCard.naturalDisasterLinkLabel'),
    href: t('claims.legalCard.naturalDisasterLinkHref'),
  },
])

const deadlineReminderClaims = computed(() =>
  allClaims.value
    .filter(claim => !claim.transmitted_at && claim.status === 'open' && (claim.is_deadline_approaching || claim.is_deadline_missed || claim.days_remaining === 0))
    .slice()
    .sort((left, right) => left.legal_deadline_at.localeCompare(right.legal_deadline_at)),
)

function claimReminderTone(claim: Claim): 'err' | 'warn' {
  return claim.is_deadline_missed || claim.days_remaining === 0 ? 'err' : 'warn'
}

const claimReminderRows = computed(() =>
  deadlineReminderClaims.value.map(claim => ({
    id: claim.id,
    title: claim.title,
    subtitle: `${t(`claims.types.${claim.claim_type}`)} · ${unitLabel(claim.unit_id)}`,
    emphasis: deadlineLabel(claim),
    tone: claimReminderTone(claim),
  })),
)

function statusVariant(status: ClaimStatus): 'open' | 'pending' | 'closed' | 'default' {
  if (status === 'open') return 'pending'
  if (status === 'in_progress') return 'open'
  if (status === 'closed') return 'closed'
  return 'default'
}

function typeIcon(type: ClaimType) {
  const icons = {
    current: CIconShield,
    natural_disaster: CIconHome,
    theft: CIconLockOpen,
  }

  return icons[type] ?? CIconShield
}

function typeTone(type: ClaimType): string {
  const tones = {
    current: 'tone--accent',
    natural_disaster: 'tone--warn',
    theft: 'tone--err',
  }

  return tones[type] ?? 'tone--default'
}

function unitLabel(unitId?: string): string {
  if (!unitId) return t('common.commonAreas')
  return t('common.unitLabel', { number: unitsById.value.get(unitId)?.unit_number ?? '—' })
}

function deadlineVariant(claim: Claim): 'valid' | 'pending' | 'err' | 'default' {
  if (claim.transmitted_at) return claim.is_transmitted_late ? 'err' : 'valid'
  if (claim.is_deadline_missed) return 'err'
  if (claim.is_deadline_approaching) return 'pending'
  return 'default'
}

function deadlineLabel(claim: Claim): string {
  if (claim.transmitted_at) {
    return t(
      claim.is_transmitted_late
        ? 'claims.deadline.transmittedLate'
        : 'claims.deadline.transmittedOnTime',
      {
        date: formatDate(claim.transmitted_at),
        deadline: formatDate(claim.legal_deadline_at),
      },
    )
  }

  if (claim.is_deadline_missed) {
    return t('claims.deadline.missed', { date: formatDate(claim.legal_deadline_at) })
  }

  if (claim.days_remaining === 0) {
    return t('claims.deadline.today', { date: formatDate(claim.legal_deadline_at) })
  }

  if (typeof claim.days_remaining === 'number') {
    return t('claims.deadline.remaining', {
      days: claim.days_remaining,
      date: formatDate(claim.legal_deadline_at),
    })
  }

  return t('claims.deadline.reference', { date: formatDate(claim.legal_deadline_at) })
}

function reimbursementLabel(claim: Claim): string {
  if (claim.reimbursement_amount != null && claim.reimbursement_date) {
    return t('claims.reimbursement.dateAndAmount', {
      amount: formatCurrency(claim.reimbursement_amount),
      date: formatDate(claim.reimbursement_date),
    })
  }

  if (claim.reimbursement_amount != null) {
    return t('claims.reimbursement.amountOnly', {
      amount: formatCurrency(claim.reimbursement_amount),
    })
  }

  if (claim.reimbursement_date) {
    return t('claims.reimbursement.dateOnly', {
      date: formatDate(claim.reimbursement_date),
    })
  }

  return t('claims.reimbursement.none')
}

function openCreateModal() {
  showCreateModal.value = true
}

function openEditModal(claim: Claim) {
  claimToEdit.value = claim
}

function openTransmitModal(claim: Claim) {
  claimToTransmit.value = claim
}

function openCloseModal(claim: Claim) {
  claimToClose.value = claim
}

async function openClaimDocuments(claim: Claim) {
  claimDocumentsTarget.value = claim
  claimDocuments.value = []
  loadingClaimDocuments.value = true

  try {
    const response = await claimsApi.listDocuments(claim.id)
    claimDocuments.value = response.data
  } catch {
    claimDocumentsTarget.value = null
  } finally {
    loadingClaimDocuments.value = false
  }
}

async function downloadLinkedDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch {
    // intentionally silent
  }
}

function handleEditDeleteRequested() {
  const claim = claimToEdit.value
  claimToEdit.value = null
  if (claim) claimToDelete.value = claim
}

function handleDocumentCountChanged(claimId: string, delta: number) {
  const claim = allClaims.value.find(c => c.id === claimId)
  if (claim) claim.document_count = Math.max(0, claim.document_count + delta)
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName || '…' }}
          <CIconChevronRight />
          <span>{{ t('common.claims') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('claims.create')"
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
            {{ t('page.eyebrows.claims') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('common.claims') }}
          </CHeading>
          <CMetaItem variant="sub">
            {{ t('claims.activeCount', { count: activeClaimsCount }) }}
          </CMetaItem>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
          </p>
        </header>

        <CReminderCard
          v-if="claimReminderRows.length > 0"
          :title="t('claims.upcomingReminders')"
          :rows="claimReminderRows"
        >
          <template
            v-if="canManage"
            #actions="{ row }"
          >
            <CBtn
              variant="negative"
              size="sm"
              :icon="CIconUpload"
              :title="t('claims.transmit')"
              @click="openTransmitModal(claimsById.get(row.id)!)"
            />
          </template>
        </CReminderCard>

        <CTabs
          v-model="filter"
          variant="filter"
          :tabs="claimTabsList"
        />

        
        <template v-if="claims.length === 0">
          <CEmptyState
            :title="t('claims.empty')"
          />  
        </template>
        <template v-else>
          <div class="flex flex-col gap-3">
            <CCard
              v-for="claim in claims"
              :key="claim.id"
              :shadow="claim.status !== 'closed'"
              gap="sm"
              :class="[{ 'outline outline-1 outline-outline-variant': claim.status === 'closed' }]"
            >
              <div class="flex items-center gap-3 min-w-0">
                <CIconTile
                  :class="typeTone(claim.claim_type)"
                >
                  <component
                    :is="typeIcon(claim.claim_type)"
                    aria-hidden="true"
                  />
                </CIconTile>
                <CMetaLine class="text-xs justify-end justify-self-end">
                  <CBadge
                    :variant="statusVariant(claim.status)"
                    dot
                  >
                    {{ t(`claims.statuses.${claim.status}`) }}
                  </CBadge>
                  <CBadge variant="default">
                    {{ t(`claims.types.${claim.claim_type}`) }}
                  </CBadge>
                  <CBadge
                    :variant="deadlineVariant(claim)"
                    :dot="deadlineVariant(claim) !== 'default'"
                  >
                    {{ deadlineLabel(claim) }}
                  </CBadge>
                </CMetaLine>
              </div>
              <div class="flex flex-col gap-3">
                <CFlexGroup
                  direction="row"
                >
                  <h3 class="text-lg font-semibold text-on-surface m-0 leading-snug min-w-0">
                    {{ claim.title }}
                  </h3>
                  <CMetaItem
                    variant="sub"
                  >
                    <template #icon>
                      <CIconLocation />
                    </template>
                    {{ unitLabel(claim.unit_id) }}
                  </CMetaItem>
                  <CMetaItem
                    v-if="claim.document_count > 0"
                    variant="sub"
                    as="button"
                    @click="openClaimDocuments(claim)"
                  >
                    <template #icon>
                      <CIconFile />
                    </template>
                    {{ t('common.documentsCount', claim.document_count) }}
                  </CMetaItem>
                </CFlexGroup>
                <p
                  v-if="claim.description"
                >
                  {{ claim.description }}
                </p>
                <template v-if="canManage">
                  <CCardActions>
                    <CBtn
                      v-if="!claim.transmitted_at"
                      variant="ghost"
                      size="sm"
                      :icon="CIconEdit"
                      :title="t('common.edit')"
                      @click="openEditModal(claim)"
                    />
                    <CBtn
                      v-if="claim.status === 'open'"
                      variant="primary"
                      size="sm"
                      :icon="CIconUpload"
                      :title="t('claims.transmit')"
                      @click="openTransmitModal(claim)"
                    />
                    <CBtn
                      v-else-if="claim.status === 'in_progress'"
                      variant="primary"
                      size="sm"
                      :icon="CIconCheckCircle"
                      :title="t('claims.close')"
                      @click="openCloseModal(claim)"
                    />
                  </CCardActions>
                </template>
                <CCardNote>
                  <span class="inline-flex items-center gap-1 w-fit">
                    <CIconCalendar />
                    {{ t('claims.declaredOn', { date: formatDate(claim.declared_at) }) }}
                  </span>
                  <span class="inline-flex items-center gap-1 w-fit">
                    <CIconClock
                      
                      class="icon-xs"
                    />
                    {{ t('claims.deadline.reference', { date: formatDate(claim.legal_deadline_at) }) }}
                  </span>
                  <span
                    v-if="claim.published_at"
                    class="inline-flex items-center gap-1 w-fit"
                  >
                    <CIconXCircle
                      class="icon-xs"
                    />
                    {{ t('claims.publishedOn', { date: formatDate(claim.published_at) }) }}
                  </span>
                  <span
                    v-if="claim.reimbursement_date || claim.reimbursement_amount != null"
                    class="inline-flex items-center gap-1 w-fit"
                  >
                    <CIconCurrency
                      
                      class="icon-xs"
                    />
                    {{ reimbursementLabel(claim) }}
                  </span>
                </CCardNote>
              </div>
            </CCard>
          </div>
        </template>

        <CLegalCard>
          <CLegalNotice :links="legalCardLinks">
            <CLegalDescription
              :title="t('claims.legalCard.title')"
              :summary="t('claims.legalCard.summary')"
            />
          </CLegalNotice>
        </CLegalCard>
      </template>
    </div>

    <ClaimCreateModal
      :open="showCreateModal"
      :units="allUnits"
      @update:open="showCreateModal = $event"
      @created="reload"
    />

    <ClaimEditModal
      :open="!!claimToEdit"
      :claim="claimToEdit"
      :units="allUnits"
      @update:open="value => !value && (claimToEdit = null)"
      @updated="reload"
      @delete-requested="handleEditDeleteRequested"
      @document-count-changed="handleDocumentCountChanged"
    />

    <ClaimTransmitModal
      :open="!!claimToTransmit"
      :claim="claimToTransmit"
      @update:open="value => !value && (claimToTransmit = null)"
      @transmitted="reload"
    />

    <ClaimCloseModal
      :open="!!claimToClose"
      :claim="claimToClose"
      @update:open="value => !value && (claimToClose = null)"
      @closed="reload"
    />

    <ClaimDeleteModal
      :open="!!claimToDelete"
      :claim="claimToDelete"
      @update:open="value => !value && (claimToDelete = null)"
      @deleted="reload"
    />

    <CLinkedDocumentsModal
      :open="!!claimDocumentsTarget"
      :title="claimDocumentsTarget?.title ?? ''"
      :documents="claimDocuments"
      :loading="loadingClaimDocuments"
      :empty-message="t('claims.noAttachedDocuments')"
      @update:open="value => !value && (claimDocumentsTarget = null)"
      @download="downloadLinkedDocument"
    />
  </div>
</template>
