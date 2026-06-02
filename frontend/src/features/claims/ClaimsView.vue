<script setup lang="ts">
import { computed, onMounted, ref, watch, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useRole } from '@/composables/useRole'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CForm from '@/components/molecules/CForm.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CDocumentLinkActionModal from '@/components/organisms/CDocumentLinkActionModal.vue'
import CLinkedDocumentsModal from '@/components/organisms/CLinkedDocumentsModal.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CCard from '@/components/molecules/CCard.vue'
import CCardActions from '@/components/molecules/CCardActions.vue'
import CCardNote from '@/components/molecules/CCardNote.vue'
import CModal from '@/components/molecules/CModal.vue'
import CReminderCard from '@/components/organisms/CReminderCard.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs, { type Tab } from '@/components/atoms/CTabs.vue'
import CIconCheckCircle from '@/components/atoms/CIconCheckCircle.svg'
import CIconClock from '@/components/atoms/CIconClock.svg'
import CIconCurrency from '@/components/atoms/CIconCurrency.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconFile from '@/components/atoms/CIconFile.svg'
import CIconHome from '@/components/atoms/CIconHome.svg'
import CIconLockOpen from '@/components/atoms/CIconLockOpen.svg'
import CIconShield from '@/components/atoms/CIconShield.svg'
import CIconXCircle from '@/components/atoms/CIconXCircle.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconUpload from '@/components/atoms/CIconUpload.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import { buildingApi, claimsApi, documentsApi, unitsApi, usersApi } from '@/api'
import { extractFileExtension } from '@/utils/files'
import type { Claim, ClaimStatus, ClaimType, Document, Unit, User } from '@/types/domain'

const { t } = useI18n()
const { formatCurrency, formatDate, formatFileSize } = useFormatters()
const { hasAnyRole } = useRole()

const CLAIM_TYPES: ClaimType[] = ['current', 'natural_disaster', 'theft']

type ClaimEditorForm = {
  title: string
  description: string
  claim_type: ClaimType
  declared_at: string
  published_at: string
  unit_id: string
}

type ClaimUpdatePayload = {
  title: string
  description?: string | null
  claim_type: string
  declared_at: string
  published_at?: string | null
  status: string
  unit_id?: string | null
  transmitted_at?: string | null
  reimbursement_date?: string | null
  reimbursement_amount?: string | null
}

const buildingName = ref('')
const allClaims = ref<Claim[]>([])
const allUnits = ref<Unit[]>([])
const allUsers = ref<User[]>([])
const filter = ref<'all' | ClaimStatus>('all')
const canManage = computed(() => hasAnyRole(['manager', 'board_member']))

const showCreateModal = ref(false)
const createFeedback = useFormFeedback()
const savingClaim = ref(false)
const uploadingDocuments = ref(false)
const selectedDocuments = ref<File[]>([])
const documentInputRef = ref<HTMLInputElement | null>(null)
const documentsDragOver = ref(false)
const createForm = ref(defaultClaimEditorForm())
const { schedule: scheduleCreateClose, forceExpire: forceCreateClose } = useAutoClose()
watch(showCreateModal, (open) => { if (!open) forceCreateClose() })

const claimToEdit = ref<Claim | null>(null)
const editFeedback = useFormFeedback()
const editingClaim = ref(false)
const uploadingEditDocuments = ref(false)
const editForm = ref(defaultClaimEditorForm())
const editDocuments = ref<Document[]>([])
const loadingEditDocuments = ref(false)
const selectedEditDocuments = ref<File[]>([])
const editDocumentInputRef = ref<HTMLInputElement | null>(null)
const editDocumentsDragOver = ref(false)
const detachingDocumentId = ref<string | null>(null)
const documentActionFeedback = useFormFeedback()
const pendingDocumentAction = ref<Document | null>(null)
const { schedule: scheduleEditClose, forceExpire: forceEditClose } = useAutoClose()
watch(claimToEdit, (claim) => { if (!claim) forceEditClose() })

const claimToTransmit = ref<Claim | null>(null)
const transmitFeedback = useFormFeedback()
const transmittingClaim = ref(false)
const transmitForm = ref({ transmitted_at: todayIso() })
const { schedule: scheduleTransmitClose, forceExpire: forceTransmitClose } = useAutoClose()
watch(claimToTransmit, (claim) => { if (!claim) forceTransmitClose() })

const claimToClose = ref<Claim | null>(null)
const closeFeedback = useFormFeedback()
const closingClaim = ref(false)
const uploadingCloseDocuments = ref(false)
const closeForm = ref({
  transmitted_at: todayIso(),
  reimbursement_date: '',
  reimbursement_amount: '',
})
const closeSelectedDocuments = ref<File[]>([])
const closeDocumentInputRef = ref<HTMLInputElement | null>(null)
const closeDocumentsDragOver = ref(false)
const { schedule: scheduleCloseClaim, forceExpire: forceCloseClaim } = useAutoClose()
watch(claimToClose, (claim) => { if (!claim) forceCloseClaim() })

const claimToDelete = ref<Claim | null>(null)
const deleteFeedback = useFormFeedback()
const deletingClaim = ref(false)
const claimDocumentsTarget = ref<Claim | null>(null)
const claimDocuments = ref<Document[]>([])
const loadingClaimDocuments = ref(false)
const { schedule: scheduleDeleteClose, forceExpire: forceDeleteClose } = useAutoClose()
watch(claimToDelete, (claim) => { if (!claim) forceDeleteClose() })

function todayIso(): string {
  return new Date().toISOString().slice(0, 10)
}

function defaultClaimEditorForm(): ClaimEditorForm {
  return {
    title: '',
    description: '',
    claim_type: 'current',
    declared_at: todayIso(),
    published_at: '',
    unit_id: '',
  }
}

function claimEditorFormFromClaim(claim: Claim): ClaimEditorForm {
  return {
    title: claim.title,
    description: claim.description ?? '',
    claim_type: claim.claim_type,
    declared_at: claim.declared_at,
    published_at: claim.published_at ?? '',
    unit_id: claim.unit_id ?? '',
  }
}

async function loadData() {
  const [buildingResponse, claimsResponse, unitsResponse, usersResponse] = await Promise.all([
    buildingApi.get(),
    claimsApi.list(),
    unitsApi.list(),
    usersApi.list(),
  ])

  buildingName.value = buildingResponse.data.name
  allClaims.value = claimsResponse.data
  allUnits.value = unitsResponse.data
  allUsers.value = usersResponse.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

const unitsById = computed(() =>
  new Map(allUnits.value.map(unit => [unit.id, unit])),
)

const claimTypeOptions = computed(() =>
  CLAIM_TYPES.map(claimType => ({
    value: claimType,
    label: t(`claims.types.${claimType}`),
  })),
)

const unitOptions = computed(() => [
  { value: '', label: t('claims.commonAreas') },
  ...allUnits.value.map(unit => ({
    value: unit.id,
    label: t('claims.unitLabel', { number: unit.unit_number }),
  })),
])

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
  if (!unitId) return t('claims.commonAreas')
  return t('claims.unitLabel', { number: unitsById.value.get(unitId)?.unit_number ?? '—' })
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

function appendDocuments(target: Ref<File[]>, files: File[]) {
  if (files.length === 0) return
  target.value = [...target.value, ...files]
}

function handleDocumentChange(
  event: Event,
  target: Ref<File[]>,
  inputRef: Ref<HTMLInputElement | null>,
) {
  const input = event.target as HTMLInputElement
  appendDocuments(target, Array.from(input.files ?? []))
  if (inputRef.value) inputRef.value.value = ''
}

function handleDocumentDrop(
  event: DragEvent,
  target: Ref<File[]>,
  dragState: Ref<boolean>,
) {
  dragState.value = false
  appendDocuments(target, Array.from(event.dataTransfer?.files ?? []))
}

function removeDocument(target: Ref<File[]>, index: number) {
  target.value = target.value.filter((_, currentIndex) => currentIndex !== index)
}

function onCreateDocumentChange(event: Event) {
  handleDocumentChange(event, selectedDocuments, documentInputRef)
}

function onCreateDocumentDrop(event: DragEvent) {
  handleDocumentDrop(event, selectedDocuments, documentsDragOver)
}

function removeCreateDocument(index: number) {
  removeDocument(selectedDocuments, index)
}

function onCloseDocumentChange(event: Event) {
  handleDocumentChange(event, closeSelectedDocuments, closeDocumentInputRef)
}

function onCloseDocumentDrop(event: DragEvent) {
  handleDocumentDrop(event, closeSelectedDocuments, closeDocumentsDragOver)
}

function removeCloseDocument(index: number) {
  removeDocument(closeSelectedDocuments, index)
}

function onEditDocumentChange(event: Event) {
  handleDocumentChange(event, selectedEditDocuments, editDocumentInputRef)
}

function onEditDocumentDrop(event: DragEvent) {
  handleDocumentDrop(event, selectedEditDocuments, editDocumentsDragOver)
}

function removeEditDocument(index: number) {
  removeDocument(selectedEditDocuments, index)
}

function normalizePublishedAt(claimType: ClaimType, publishedAt: string): string | null {
  if (claimType !== 'natural_disaster') return null
  return publishedAt || null
}

function parseOptionalAmount(value: string): string | null {
  const normalized = value.trim().replace(',', '.')
  if (!normalized) return null
  const amount = Number(normalized)
  return Number.isFinite(amount) && amount > 0 ? normalized : null
}

function buildClaimUpdatePayload(
  claim: Claim,
  overrides: Partial<ClaimUpdatePayload>,
): ClaimUpdatePayload {
  return {
    title: claim.title,
    description: claim.description ?? null,
    claim_type: claim.claim_type,
    declared_at: claim.declared_at,
    published_at: claim.published_at ?? null,
    status: claim.status,
    unit_id: claim.unit_id ?? null,
    transmitted_at: claim.transmitted_at ?? null,
    reimbursement_date: claim.reimbursement_date ?? null,
    reimbursement_amount: claim.reimbursement_amount != null ? String(claim.reimbursement_amount) : null,
    ...overrides,
  }
}

function openCreateModal() {
  createForm.value = defaultClaimEditorForm()
  selectedDocuments.value = []
  documentsDragOver.value = false
  createFeedback.reset()
  showCreateModal.value = true
}

async function loadEditDocuments(claimId: string) {
  loadingEditDocuments.value = true

  try {
    const response = await claimsApi.listDocuments(claimId)
    editDocuments.value = response.data
  } catch (currentError: unknown) {
    editFeedback.setError(toErrorMessage(currentError, t('claims.documentsLoadError')))
  } finally {
    loadingEditDocuments.value = false
  }
}

async function openClaimDocuments(claim: Claim) {
  claimDocumentsTarget.value = claim
  claimDocuments.value = []
  loadingClaimDocuments.value = true

  try {
    const response = await claimsApi.listDocuments(claim.id)
    claimDocuments.value = response.data
  } catch (currentError: unknown) {
    editFeedback.setError(toErrorMessage(currentError, t('claims.documentsLoadError')))
    claimDocumentsTarget.value = null
  } finally {
    loadingClaimDocuments.value = false
  }
}

async function openEditModal(claim: Claim) {
  claimToEdit.value = claim
  editForm.value = claimEditorFormFromClaim(claim)
  selectedEditDocuments.value = []
  editDocumentsDragOver.value = false
  editDocuments.value = []
  editFeedback.reset()
  await loadEditDocuments(claim.id)
}

function openTransmitModal(claim: Claim) {
  claimToTransmit.value = claim
  transmitForm.value = {
    transmitted_at: claim.transmitted_at ?? todayIso(),
  }
  transmitFeedback.reset()
}

function openCloseModal(claim: Claim) {
  claimToClose.value = claim
  closeForm.value = {
    transmitted_at: claim.transmitted_at ?? todayIso(),
    reimbursement_date: claim.reimbursement_date ?? '',
    reimbursement_amount: claim.reimbursement_amount != null ? String(claim.reimbursement_amount) : '',
  }
  closeSelectedDocuments.value = []
  closeDocumentsDragOver.value = false
  closeFeedback.reset()
}

function openDeleteModal(claim: Claim) {
  claimToDelete.value = claim
  deleteFeedback.reset()
}

function openDeleteFromEdit() {
  const claim = claimToEdit.value
  claimToEdit.value = null
  if (claim) openDeleteModal(claim)
}

async function uploadDocument(claimId: string, file: File): Promise<void> {
  const extension = extractFileExtension(file.name)
  const mimeType = file.type || 'application/octet-stream'
  const uploadUrlResponse = await documentsApi.getUploadUrl(extension, mimeType, file.size)
  const { url, key, bucket } = uploadUrlResponse.data
  await documentsApi.uploadToS3(file, url, mimeType)
  const documentResponse = await documentsApi.confirmUpload({
    key,
    bucket,
    original_name: file.name,
    mime_type: mimeType,
    size_bytes: file.size,
    visible_roles: ['manager', 'board_member', 'co_owner', 'tenant'],
  })
  await claimsApi.attachDocument(claimId, documentResponse.data.id)
}

async function attachSelectedDocuments(claimId: string, documents: File[]): Promise<string[]> {
  const failedDocuments: string[] = []

  for (const document of documents) {
    try {
      await uploadDocument(claimId, document)
    } catch {
      failedDocuments.push(document.name)
    }
  }

  return failedDocuments
}

async function downloadClaimDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch (currentError: unknown) {
    editFeedback.setError(toErrorMessage(currentError, t('common.error')))
  }
}

function openDocumentActionModal(document: Document) {
  pendingDocumentAction.value = document
  documentActionFeedback.reset()
}

function closeDocumentActionModal() {
  if (detachingDocumentId.value) return
  pendingDocumentAction.value = null
  documentActionFeedback.reset()
}

function updateClaimDocumentCount(claimId: string, delta: number) {
  const claim = allClaims.value.find(currentClaim => currentClaim.id === claimId)
  if (claim) {
    claim.document_count = Math.max(0, claim.document_count + delta)
  }
}

async function handleDocumentAction(deleteDocument: boolean) {
  if (!claimToEdit.value || !pendingDocumentAction.value) return

  detachingDocumentId.value = pendingDocumentAction.value.id

  try {
    if (deleteDocument) {
      await documentsApi.delete(pendingDocumentAction.value.id)
    } else {
      await claimsApi.detachDocument(claimToEdit.value.id, pendingDocumentAction.value.id)
    }
    editDocuments.value = editDocuments.value.filter(document => document.id !== pendingDocumentAction.value?.id)
    claimDocuments.value = claimDocuments.value.filter(document => document.id !== pendingDocumentAction.value?.id)
    updateClaimDocumentCount(claimToEdit.value.id, -1)
    pendingDocumentAction.value = null
  } catch (currentError: unknown) {
    documentActionFeedback.setError(
      toErrorMessage(
        currentError,
        deleteDocument ? t('claims.documentsDeleteError') : t('claims.documentsDetachError'),
      ),
    )
  } finally {
    detachingDocumentId.value = null
  }
}

async function submitCreate() {
  if (createFeedback.status.value === 'success') {
    forceCreateClose()
    return
  }

  savingClaim.value = true
  createFeedback.reset()

  try {
    const response = await claimsApi.create({
      title: createForm.value.title.trim(),
      description: createForm.value.description.trim() || undefined,
      claim_type: createForm.value.claim_type,
      declared_at: createForm.value.declared_at,
      published_at: normalizePublishedAt(createForm.value.claim_type, createForm.value.published_at) ?? undefined,
      unit_id: createForm.value.unit_id || undefined,
    })

    if (selectedDocuments.value.length > 0) {
      uploadingDocuments.value = true
      const failedDocuments = await attachSelectedDocuments(response.data.id, selectedDocuments.value)
      uploadingDocuments.value = false

      if (failedDocuments.length > 0) {
        createFeedback.setError(
          `${t('claims.documentUploadError')}: ${failedDocuments.join(', ')}`,
        )
        setTimeout(() => {
          showCreateModal.value = false
          reload()
        }, 2200)
        return
      }
    }

    createFeedback.setSuccess(t('claims.createSuccess'))
    scheduleCreateClose(() => {
      showCreateModal.value = false
      reload()
    })
  } catch (currentError: unknown) {
    createFeedback.setError(toErrorMessage(currentError, t('common.error')))
  } finally {
    savingClaim.value = false
    uploadingDocuments.value = false
  }
}

async function submitEdit() {
  if (editFeedback.status.value === 'success') {
    forceEditClose()
    return
  }

  if (!claimToEdit.value) return

  editingClaim.value = true
  editFeedback.reset()

  try {
    await claimsApi.update(
      claimToEdit.value.id,
      buildClaimUpdatePayload(claimToEdit.value, {
        title: editForm.value.title.trim(),
        description: editForm.value.description.trim() || null,
        claim_type: editForm.value.claim_type,
        declared_at: editForm.value.declared_at,
        published_at: normalizePublishedAt(editForm.value.claim_type, editForm.value.published_at),
        unit_id: editForm.value.unit_id || null,
      }),
    )

    if (selectedEditDocuments.value.length > 0) {
      uploadingEditDocuments.value = true
      const failedDocuments = await attachSelectedDocuments(
        claimToEdit.value.id,
        selectedEditDocuments.value,
      )
      uploadingEditDocuments.value = false

      if (failedDocuments.length > 0) {
        editFeedback.setError(
          `${t('claims.documentUploadError')}: ${failedDocuments.join(', ')}`,
        )
        setTimeout(() => {
          claimToEdit.value = null
          reload()
        }, 2200)
        return
      }
    }

    editFeedback.setSuccess(t('claims.updateSuccess'))
    scheduleEditClose(() => {
      claimToEdit.value = null
      reload()
    })
  } catch (currentError: unknown) {
    editFeedback.setError(toErrorMessage(currentError, t('common.error')))
  } finally {
    editingClaim.value = false
    uploadingEditDocuments.value = false
  }
}

async function submitTransmit() {
  if (transmitFeedback.status.value === 'success') {
    forceTransmitClose()
    return
  }

  if (!claimToTransmit.value) return
  if (!transmitForm.value.transmitted_at) {
    transmitFeedback.setError(t('claims.transmittedAtRequired'))
    return
  }

  transmittingClaim.value = true
  transmitFeedback.reset()

  try {
    await claimsApi.update(
      claimToTransmit.value.id,
      buildClaimUpdatePayload(claimToTransmit.value, {
        status: 'in_progress',
        transmitted_at: transmitForm.value.transmitted_at,
      }),
    )

    transmitFeedback.setSuccess(t('claims.transmitSuccess'))
    scheduleTransmitClose(() => {
      claimToTransmit.value = null
      reload()
    })
  } catch (currentError: unknown) {
    transmitFeedback.setError(toErrorMessage(currentError, t('common.error')))
  } finally {
    transmittingClaim.value = false
  }
}

async function submitClose() {
  if (closeFeedback.status.value === 'success') {
    forceCloseClaim()
    return
  }

  if (!claimToClose.value) return
  if (!closeForm.value.transmitted_at) {
    closeFeedback.setError(t('claims.transmittedAtRequired'))
    return
  }

  const reimbursementAmountInput = closeForm.value.reimbursement_amount.trim()
  const reimbursementAmount = parseOptionalAmount(reimbursementAmountInput)
  if (reimbursementAmountInput && reimbursementAmount == null) {
    closeFeedback.setError(t('claims.invalidReimbursementAmount'))
    return
  }

  closingClaim.value = true
  closeFeedback.reset()

  try {
    await claimsApi.update(
      claimToClose.value.id,
      buildClaimUpdatePayload(claimToClose.value, {
        status: 'closed',
        transmitted_at: closeForm.value.transmitted_at,
        reimbursement_date: closeForm.value.reimbursement_date || null,
        reimbursement_amount: reimbursementAmount,
      }),
    )

    if (closeSelectedDocuments.value.length > 0) {
      uploadingCloseDocuments.value = true
      const failedDocuments = await attachSelectedDocuments(
        claimToClose.value.id,
        closeSelectedDocuments.value,
      )
      uploadingCloseDocuments.value = false

      if (failedDocuments.length > 0) {
        closeFeedback.setError(
          `${t('claims.closeDocumentUploadError')}: ${failedDocuments.join(', ')}`,
        )
        setTimeout(() => {
          claimToClose.value = null
          reload()
        }, 2200)
        return
      }
    }

    closeFeedback.setSuccess(t('claims.closeSuccess'))
    scheduleCloseClaim(() => {
      claimToClose.value = null
      reload()
    })
  } catch (currentError: unknown) {
    closeFeedback.setError(toErrorMessage(currentError, t('common.error')))
  } finally {
    closingClaim.value = false
    uploadingCloseDocuments.value = false
  }
}

async function submitDelete() {
  if (deleteFeedback.status.value === 'success') {
    forceDeleteClose()
    return
  }

  if (!claimToDelete.value) return

  deletingClaim.value = true
  deleteFeedback.reset()

  try {
    await claimsApi.delete(claimToDelete.value.id)
    deleteFeedback.setSuccess(t('claims.deleteSuccess'))
    scheduleDeleteClose(() => {
      claimToDelete.value = null
      reload()
    })
  } catch (currentError: unknown) {
    deleteFeedback.setError(toErrorMessage(currentError, t('common.error')))
  } finally {
    deletingClaim.value = false
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName || '…' }}
          <CIconChevronRight />
          <span>{{ t('claims.title') }}</span>
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
            {{ t('claims.title') }}
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
                    {{ t('claims.documentsCount', claim.document_count) }}
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

    <CModal
      :open="showCreateModal"
      :title="t('claims.create')"
      size="lg"
      :closable="!savingClaim && !uploadingDocuments"
      @update:open="showCreateModal = $event"
    >
      <CForm
        @submit="submitCreate"
      >
        <CFormInput
          v-model="createForm.title"
          :label="t('claims.fields.title')"
          required
        />
        <CFormSelect
          v-model="createForm.claim_type"
          :label="t('claims.fields.claimType')"
          :options="claimTypeOptions"
        />

        <div class="form-row">
          <CFormInput
            v-model="createForm.declared_at"
            :label="t('claims.fields.declaredAt')"
            type="date"
            required
          />
          <CFormInput
            v-if="createForm.claim_type === 'natural_disaster'"
            v-model="createForm.published_at"
            :label="t('claims.fields.publishedAt')"
            type="date"
            required
            :hint="t('claims.fields.publishedAtHint')"
          />
        </div>

        <CFormSelect
          v-model="createForm.unit_id"
          :label="t('claims.fields.unit')"
          :options="unitOptions"
        />
        <CFormTextarea
          v-model="createForm.description"
          :label="t('claims.fields.description')"
          :rows="3"
        />

        <div class="flex flex-col gap-1">
          <label class="text-sm font-medium text-on-surface-variant">{{ t('claims.fields.documents') }}</label>
          <input
            ref="documentInputRef"
            type="file"
            multiple
            style="display:none"
            @change="onCreateDocumentChange"
          />
          <div
            :class="['dropzone', { 'dropzone--over': documentsDragOver }]"
            role="button"
            tabindex="0"
            @click="documentInputRef?.click()"
            @keydown.enter.prevent="documentInputRef?.click()"
            @keydown.space.prevent="documentInputRef?.click()"
            @dragenter.prevent="documentsDragOver = true"
            @dragover.prevent="documentsDragOver = true"
            @dragleave.prevent="documentsDragOver = false"
            @drop.prevent="onCreateDocumentDrop"
          >
            <CIconUpload class="dropzone__icon" />
            <span class="body-xs dropzone__hint">{{ t('claims.documentHint') }}</span>
          </div>

          <div
            v-if="selectedDocuments.length > 0"
            class="claim-preview-strip"
          >
            <div
              v-for="(document, index) in selectedDocuments"
              :key="`${document.name}-${index}`"
              class="claim-preview-item"
            >
              <span class="claim-preview-name">{{ document.name }}</span>
              <button
                type="button"
                class="claim-remove"
                :title="t('claims.removeDocument')"
                :aria-label="t('claims.removeDocument')"
                @click="removeCreateDocument(index)"
              >
                ×
              </button>
            </div>
          </div>
        </div>
      </CForm>

      <template #footer>
        <CFormActions
          :status="createFeedback.status.value"
          :message="createFeedback.message.value"
        >
          <CBtn
            variant="ghost"
            :title="t('common.cancel')"
            @click="showCreateModal = false"
          />
          <CBtn
            variant="primary"
            :title="t('common.save')"
            :loading="savingClaim || uploadingDocuments"
            :closing="createFeedback.status.value === 'success'"
            @click="submitCreate"
          />
        </CFormActions>
      </template>
    </CModal>

    <CModal
      :open="!!claimToEdit"
      :title="t('claims.editTitle')"
      size="lg"
      @update:open="value => !value && (claimToEdit = null)"
    >
      <CForm
        @submit="submitEdit"
      >
        <CFormInput
          v-model="editForm.title"
          :label="t('claims.fields.title')"
          required
        />
        <CFormSelect
          v-model="editForm.claim_type"
          :label="t('claims.fields.claimType')"
          :options="claimTypeOptions"
        />

        <div class="form-row">
          <CFormInput
            v-model="editForm.declared_at"
            :label="t('claims.fields.declaredAt')"
            type="date"
            required
          />
          <CFormInput
            v-if="editForm.claim_type === 'natural_disaster'"
            v-model="editForm.published_at"
            :label="t('claims.fields.publishedAt')"
            type="date"
            required
            :hint="t('claims.fields.publishedAtHint')"
          />
        </div>

        <CFormSelect
          v-model="editForm.unit_id"
          :label="t('claims.fields.unit')"
          :options="unitOptions"
        />
        <CFormTextarea
          v-model="editForm.description"
          :label="t('claims.fields.description')"
          :rows="3"
        />

        <div class="flex flex-col gap-1">
          <label class="text-sm font-medium text-on-surface-variant">{{ t('claims.fields.documents') }}</label>
          <input
            ref="editDocumentInputRef"
            type="file"
            multiple
            style="display:none"
            @change="onEditDocumentChange"
          />
          <div
            :class="['dropzone', { 'dropzone--over': editDocumentsDragOver }]"
            role="button"
            tabindex="0"
            @click="editDocumentInputRef?.click()"
            @keydown.enter.prevent="editDocumentInputRef?.click()"
            @keydown.space.prevent="editDocumentInputRef?.click()"
            @dragenter.prevent="editDocumentsDragOver = true"
            @dragover.prevent="editDocumentsDragOver = true"
            @dragleave.prevent="editDocumentsDragOver = false"
            @drop.prevent="onEditDocumentDrop"
          >
            <CIconUpload class="dropzone__icon" />
            <span class="body-xs dropzone__hint">{{ t('claims.editDocumentHint') }}</span>
          </div>

          <div
            v-if="selectedEditDocuments.length > 0"
            class="claim-preview-strip"
          >
            <div
              v-for="(document, index) in selectedEditDocuments"
              :key="`${document.name}-${index}`"
              class="claim-preview-item"
            >
              <span class="claim-preview-name">{{ document.name }}</span>
              <button
                type="button"
                class="claim-remove"
                :title="t('claims.removeDocument')"
                :aria-label="t('claims.removeDocument')"
                @click="removeEditDocument(index)"
              >
                ×
              </button>
            </div>
          </div>

          <div class="claim-documents-section">
            <p class="text-xs text-on-surface-variant claim-documents-section__hint">
              {{ t('claims.fields.attachedDocuments') }}
            </p>
            <CPageLoader v-if="loadingEditDocuments" />
            <div
              v-else-if="editDocuments.length === 0"
              class="claim-documents-empty"
            >
              {{ t('claims.noAttachedDocuments') }}
            </div>
            <div
              v-else
              class="claim-documents-list"
            >
              <div
                v-for="document in editDocuments"
                :key="document.id"
                class="claim-documents-row"
              >
                <div class="claim-documents-meta">
                  <div class="claim-documents-name">
                    <CIconFile
                      class="icon-sm"
                    />
                    <span>{{ document.original_name }}</span>
                  </div>
                  <div class="claim-documents-details">
                    <span>{{ formatFileSize(document.size_bytes) }}</span>
                    <span>·</span>
                    <span>{{ formatDate(document.created_at) }}</span>
                  </div>
                </div>
                <div class="claim-documents-actions">
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :title="t('common.download')"
                    @click="downloadClaimDocument(document.id)"
                  />
                  <CBtn
                    variant="danger"
                    size="sm"
                    :title="t('claims.detachDocument')"
                    :loading="detachingDocumentId === document.id"
                    @click="openDocumentActionModal(document)"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </CForm>

      <template #footer>
        <CFormActions
          :status="editFeedback.status.value"
          :message="editFeedback.message.value"
        >
          <CBtn
            v-if="claimToEdit"
            variant="danger"
            :icon="CIconTrash"
            :title="t('common.delete')"
            class="mr-auto"
            @click="openDeleteFromEdit"
          />
          <CBtn
            variant="ghost"
            :title="t('common.cancel')"
            @click="claimToEdit = null"
          />
          <CBtn
            variant="primary"
            :title="t('common.save')"
            :loading="editingClaim || uploadingEditDocuments"
            :closing="editFeedback.status.value === 'success'"
            @click="submitEdit"
          />
        </CFormActions>
      </template>
    </CModal>

    <CModal
      :open="!!claimToTransmit"
      :title="t('claims.transmitTitle')"
      size="sm"
      @update:open="value => !value && (claimToTransmit = null)"
    >
      <CForm
        @submit="submitTransmit"
      >
        <CFormInput
          v-model="transmitForm.transmitted_at"
          :label="t('claims.fields.transmittedAt')"
          type="date"
          required
          :hint="t('claims.fields.transmittedAtHint')"
        />
      </CForm>

      <template #footer>
        <CFormActions
          :status="transmitFeedback.status.value"
          :message="transmitFeedback.message.value"
        >
          <CBtn
            variant="ghost"
            @click="claimToTransmit = null"
          >
            {{ t('common.cancel') }}
          </CBtn>
          <CBtn
            variant="primary"
            :loading="transmittingClaim"
            :closing="transmitFeedback.status.value === 'success'"
            @click="submitTransmit"
          >
            {{ t('claims.transmit') }}
          </CBtn>
        </CFormActions>
      </template>
    </CModal>

    <CModal
      :open="!!claimToClose"
      :title="t('claims.closeTitle')"
      size="lg"
      @update:open="value => !value && (claimToClose = null)"
    >
      <CForm
        @submit="submitClose"
      >
        <CFormInput
          v-model="closeForm.transmitted_at"
          :label="t('claims.fields.transmittedAt')"
          type="date"
          required
          :hint="t('claims.fields.transmittedAtHint')"
        />

        <div class="form-row">
          <CFormInput
            v-model="closeForm.reimbursement_date"
            :label="t('claims.fields.reimbursementDate')"
            type="date"
          />
          <CFormInput
            v-model="closeForm.reimbursement_amount"
            :label="t('claims.fields.reimbursementAmount')"
            type="number"
            :min="0.01"
            :step="0.01"
            inputmode="decimal"
            :hint="t('claims.fields.reimbursementAmountHint')"
          />
        </div>

        <div class="flex flex-col gap-1">
          <label class="text-sm font-medium text-on-surface-variant">{{ t('claims.fields.closeDocuments') }}</label>
          <input
            ref="closeDocumentInputRef"
            type="file"
            multiple
            style="display:none"
            @change="onCloseDocumentChange"
          />
          <div
            :class="['dropzone', { 'dropzone--over': closeDocumentsDragOver }]"
            role="button"
            tabindex="0"
            @click="closeDocumentInputRef?.click()"
            @keydown.enter.prevent="closeDocumentInputRef?.click()"
            @keydown.space.prevent="closeDocumentInputRef?.click()"
            @dragenter.prevent="closeDocumentsDragOver = true"
            @dragover.prevent="closeDocumentsDragOver = true"
            @dragleave.prevent="closeDocumentsDragOver = false"
            @drop.prevent="onCloseDocumentDrop"
          >
            <CIconUpload class="dropzone__icon" />
            <span class="body-xs dropzone__hint">{{ t('claims.closeDocumentHint') }}</span>
          </div>

          <div
            v-if="closeSelectedDocuments.length > 0"
            class="claim-preview-strip"
          >
            <div
              v-for="(document, index) in closeSelectedDocuments"
              :key="`${document.name}-${index}`"
              class="claim-preview-item"
            >
              <span class="claim-preview-name">{{ document.name }}</span>
              <button
                type="button"
                class="claim-remove"
                :title="t('claims.removeDocument')"
                :aria-label="t('claims.removeDocument')"
                @click="removeCloseDocument(index)"
              >
                ×
              </button>
            </div>
          </div>
        </div>
      </CForm>

      <template #footer>
        <CFormActions
          :status="closeFeedback.status.value"
          :message="closeFeedback.message.value"
        >
          <CBtn
            variant="ghost"
            @click="claimToClose = null"
          >
            {{ t('common.cancel') }}
          </CBtn>
          <CBtn
            variant="primary"
            :loading="closingClaim || uploadingCloseDocuments"
            :closing="closeFeedback.status.value === 'success'"
            @click="submitClose"
          >
            {{ t('claims.close') }}
          </CBtn>
        </CFormActions>
      </template>
    </CModal>

    <CModal
      :open="!!claimToDelete"
      :title="t('claims.confirmDeleteTitle')"
      size="sm"
      @update:open="value => !value && (claimToDelete = null)"
    >
      <p class="body-xs">
        {{ t('claims.confirmDeleteMessage', { title: claimToDelete?.title ?? '' }) }}
      </p>

      <template #footer>
        <CFormActions
          :status="deleteFeedback.status.value"
          :message="deleteFeedback.message.value"
        >
          <CBtn
            variant="ghost"
            @click="claimToDelete = null"
          >
            {{ t('common.cancel') }}
          </CBtn>
          <CBtn
            variant="danger"
            :loading="deletingClaim"
            :closing="deleteFeedback.status.value === 'success'"
            @click="submitDelete"
          >
            {{ t('common.deleteLabel') }}
          </CBtn>
        </CFormActions>
      </template>
    </CModal>

    <CLinkedDocumentsModal
      :open="!!claimDocumentsTarget"
      :title="claimDocumentsTarget?.title ?? ''"
      :documents="claimDocuments"
      :loading="loadingClaimDocuments"
      :empty-message="t('claims.noAttachedDocuments')"
      @update:open="value => !value && (claimDocumentsTarget = null)"
      @download="downloadClaimDocument"
    />

    <CDocumentLinkActionModal
      :open="!!pendingDocumentAction"
      :title="t('claims.documentsDetachTitle')"
      :description="t('claims.documentsDetachMessage', { name: pendingDocumentAction?.original_name ?? '', title: claimToEdit?.title ?? '' })"
      :warning="t('claims.documentsDeleteWarning', { name: pendingDocumentAction?.original_name ?? '' })"
      :loading="!!detachingDocumentId"
      :status="documentActionFeedback.status.value"
      :message="documentActionFeedback.message.value"
      @update:open="value => !value && closeDocumentActionModal()"
      @unlink="handleDocumentAction(false)"
      @unlink-and-delete="handleDocumentAction(true)"
    />
  </div>
</template>

<style scoped>
.claims-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.claim-preview-strip {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: var(--space-2);
}

.claim-preview-item {
  position: relative;
  display: inline-flex;
  align-items: center;
  min-height: 44px;
  max-width: 100%;
  padding: 0 var(--space-4) 0 var(--space-3);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-outline-variant);
  background: var(--color-surface-container-low);
}

.claim-preview-name {
  max-width: 220px;
  overflow: clip;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-on-surface-variant);
  font-size: var(--text-sm);
}

.claim-remove {
  position: absolute;
  top: -6px;
  right: -6px;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--color-error);
  color: #fff;
  border: none;
  cursor: pointer;
  font-size: var(--text-md);
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  min-height: unset;
  min-width: unset;
}

.claim-remove:hover {
  background: var(--color-error-dark, var(--color-error));
  opacity: 0.85;
}

.claim-documents-section {
  margin-top: var(--space-3);
}

.claim-documents-section__hint {
  margin-bottom: var(--space-2);
}

.claim-documents-empty {
  padding: var(--space-3);
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-low);
  color: var(--color-on-surface-variant);
}

.claim-documents-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.claim-documents-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-low);
}

.claim-documents-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.claim-documents-name {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
  font-weight: var(--fw-medium);
}

.claim-documents-name span {
  overflow: clip;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.claim-documents-details {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

.claim-documents-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.modal-footer-feedback {
  margin-right: auto;
  min-width: 0;
}

.form-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

@media (max-width: 720px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .claim-documents-row {
    flex-direction: column;
    align-items: flex-start;
  }
}

</style>
