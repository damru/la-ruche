// ── Domain constants — exact mirror of Rust/PostgreSQL enums ──────────────
// Any change here must stay consistent with the backend (backend/src/db/,
// backend/src/domain/) and PostgreSQL migrations.

import type {
  AllocationKeyType,
  BudgetStatus,
  CalendarEventCategory,
  ClaimStatus,
  ClaimType,
  ContractStatus,
  ContractType,
  EtatDateStatus,
  GovernanceRole,
  JudicialProceedingStatus,
  JudicialProceedingType,
  LevyStatus,
  MaintenanceCategory,
  MeetingAttendanceMode,
  MeetingDocumentKind,
  MeetingStatus,
  MeetingType,
  PaymentFrequency,
  PollStatus,
  ReminderStatus,
  ReminderType,
  ReportStatus,
  ResolutionStatus,
  TaskPriority,
  TaskStatus,
  UnitType,
  VotePosition,
  VoteType,
} from '@/types/domain'

// ── Roles ──
//
// Role values and role groups live in `utils/roles.ts` — do NOT add role
// constants here.

// ── Meetings ──

export const MEETING_STATUSES = ['draft', 'called', 'ongoing', 'closed'] as const satisfies readonly MeetingStatus[]
export const MEETING_STATUS_DRAFT: MeetingStatus = 'draft'
export const MEETING_STATUS_CALLED: MeetingStatus = 'called'
export const MEETING_STATUS_ONGOING: MeetingStatus = 'ongoing'
export const MEETING_STATUS_CLOSED: MeetingStatus = 'closed'
export const MEETING_TYPES = ['annual', 'extraordinary'] as const satisfies readonly MeetingType[]
export const MEETING_TYPE_ANNUAL: MeetingType = 'annual'
export const MEETING_TYPE_EXTRAORDINARY: MeetingType = 'extraordinary'
export const MEETING_DOCUMENT_KINDS = ['notice', 'attachment', 'vote_by_mail_form'] as const satisfies readonly MeetingDocumentKind[]
export const MEETING_DOCUMENT_KIND_NOTICE: MeetingDocumentKind = 'notice'
export const MEETING_DOCUMENT_KIND_ATTACHMENT: MeetingDocumentKind = 'attachment'
/** Document kinds that can be uploaded and attached to a meeting (excludes vote_by_mail_form). */
export const MEETING_UPLOADABLE_DOCUMENT_KINDS = [MEETING_DOCUMENT_KIND_NOTICE, MEETING_DOCUMENT_KIND_ATTACHMENT] as const satisfies readonly MeetingDocumentKind[]
export const MEETING_ATTENDANCE_MODES = ['present', 'proxy', 'correspondence', 'absent'] as const satisfies readonly MeetingAttendanceMode[]
export const MEETING_ATTENDANCE_MODE_PRESENT: MeetingAttendanceMode = 'present'
export const MEETING_ATTENDANCE_MODE_PROXY: MeetingAttendanceMode = 'proxy'
export const MEETING_ATTENDANCE_MODE_CORRESPONDENCE: MeetingAttendanceMode = 'correspondence'
export const MEETING_ATTENDANCE_MODE_ABSENT: MeetingAttendanceMode = 'absent'
export const VOTE_TYPES = ['simple_majority', 'absolute_majority', 'double_majority', 'unanimity'] as const satisfies readonly VoteType[]
export const VOTE_TYPE_SIMPLE_MAJORITY: VoteType = 'simple_majority'
export const VOTE_TYPE_ABSOLUTE_MAJORITY: VoteType = 'absolute_majority'
export const VOTE_TYPE_DOUBLE_MAJORITY: VoteType = 'double_majority'
export const VOTE_TYPE_UNANIMITY: VoteType = 'unanimity'
export const RESOLUTION_STATUSES = ['pending', 'passed', 'rejected', 'withdrawn'] as const satisfies readonly ResolutionStatus[]
export const RESOLUTION_STATUS_PENDING: ResolutionStatus = 'pending'
export const RESOLUTION_STATUS_PASSED: ResolutionStatus = 'passed'
export const RESOLUTION_STATUS_REJECTED: ResolutionStatus = 'rejected'
export const RESOLUTION_STATUS_WITHDRAWN: ResolutionStatus = 'withdrawn'
export const VOTE_POSITIONS = ['in_favor', 'against', 'abstain', 'non_voter'] as const satisfies readonly VotePosition[]
export const VOTE_POSITION_IN_FAVOR = 'in_favor' as const satisfies VotePosition
export const VOTE_POSITION_AGAINST = 'against' as const satisfies VotePosition
export const VOTE_POSITION_ABSTAIN = 'abstain' as const satisfies VotePosition

// ── Budget ──

export const BUDGET_STATUSES = ['draft', 'approved', 'closed'] as const satisfies readonly BudgetStatus[]
export const BUDGET_STATUS_DRAFT: BudgetStatus = 'draft'
export const BUDGET_STATUS_APPROVED: BudgetStatus = 'approved'
export const BUDGET_STATUS_CLOSED: BudgetStatus = 'closed'
export const LEVY_STATUSES = ['issued', 'paid', 'overdue'] as const satisfies readonly LevyStatus[]
export const LEVY_STATUS_ISSUED: LevyStatus = 'issued'
export const LEVY_STATUS_PAID: LevyStatus = 'paid'
export const LEVY_STATUS_OVERDUE: LevyStatus = 'overdue'

// ── Tasks ──

export const TASK_STATUSES = ['open', 'in_progress', 'awaiting_vendor', 'awaiting_validation', 'closed', 'cancelled'] as const satisfies readonly TaskStatus[]
export const TASK_STATUS_OPEN: TaskStatus = 'open'
export const TASK_STATUS_IN_PROGRESS: TaskStatus = 'in_progress'
export const TASK_STATUS_AWAITING_VENDOR: TaskStatus = 'awaiting_vendor'
export const TASK_STATUS_AWAITING_VALIDATION: TaskStatus = 'awaiting_validation'
export const TASK_STATUS_CLOSED: TaskStatus = 'closed'
export const TASK_STATUS_CANCELLED: TaskStatus = 'cancelled'
export const TASK_PRIORITIES = ['low', 'normal', 'high', 'urgent'] as const satisfies readonly TaskPriority[]
export const TASK_PRIORITY_LOW: TaskPriority = 'low'
export const TASK_PRIORITY_NORMAL: TaskPriority = 'normal'
export const TASK_PRIORITY_HIGH: TaskPriority = 'high'
export const TASK_PRIORITY_URGENT: TaskPriority = 'urgent'
export const DEFAULT_TASK_PRIORITY: TaskPriority = TASK_PRIORITY_NORMAL

// ── Claims & Reports ──

export const CLAIM_STATUSES = ['draft', 'open', 'in_progress', 'closed'] as const satisfies readonly ClaimStatus[]
export const CLAIM_STATUS_DRAFT: ClaimStatus = 'draft'
export const CLAIM_STATUS_OPEN: ClaimStatus = 'open'
export const CLAIM_STATUS_IN_PROGRESS: ClaimStatus = 'in_progress'
export const CLAIM_STATUS_CLOSED: ClaimStatus = 'closed'
export const CLAIM_TYPES = ['current', 'natural_disaster', 'theft'] as const satisfies readonly ClaimType[]

/** Claim type for natural disasters (Arrêté Cat Nat) — gates the `published_at` field. */
export const CLAIM_TYPE_NATURAL_DISASTER: ClaimType = 'natural_disaster'
export const REPORT_STATUSES = ['open', 'in_progress', 'closed'] as const satisfies readonly ReportStatus[]
export const REPORT_STATUS_OPEN: ReportStatus = 'open'
export const REPORT_STATUS_IN_PROGRESS: ReportStatus = 'in_progress'
export const REPORT_STATUS_CLOSED: ReportStatus = 'closed'

// ── Consultations ──

export const POLL_STATUSES = ['draft', 'open', 'closed'] as const satisfies readonly PollStatus[]
export const POLL_STATUS_DRAFT: PollStatus = 'draft'
export const POLL_STATUS_OPEN: PollStatus = 'open'
export const POLL_STATUS_CLOSED: PollStatus = 'closed'

// ── Contracts ──

export const CONTRACT_TYPES = ['maintenance', 'insurance', 'service', 'energy', 'water', 'cleaning', 'green_spaces', 'other'] as const satisfies readonly ContractType[]
export const CONTRACT_STATUSES = ['draft', 'active', 'terminated', 'expired'] as const satisfies readonly ContractStatus[]
export const CONTRACT_STATUS_DRAFT: ContractStatus = 'draft'
export const CONTRACT_STATUS_ACTIVE: ContractStatus = 'active'
export const CONTRACT_STATUS_TERMINATED: ContractStatus = 'terminated'
export const CONTRACT_STATUS_EXPIRED: ContractStatus = 'expired'
export const REMINDER_TYPES = ['renewal', 'payment_due', 'expiration', 'termination'] as const satisfies readonly ReminderType[]
export const REMINDER_STATUSES = ['pending', 'sent', 'acknowledged'] as const satisfies readonly ReminderStatus[]
export const REMINDER_STATUS_PENDING: ReminderStatus = 'pending'
export const REMINDER_STATUS_SENT: ReminderStatus = 'sent'
export const REMINDER_STATUS_ACKNOWLEDGED: ReminderStatus = 'acknowledged'
export const PAYMENT_FREQUENCIES = ['monthly', 'quarterly', 'biannual', 'annual'] as const satisfies readonly PaymentFrequency[]

// ── Maintenance logbook ──

export const MAINTENANCE_CATEGORIES = ['travaux', 'contrat', 'sinistre', 'autre'] as const satisfies readonly MaintenanceCategory[]
export const DEFAULT_MAINTENANCE_CATEGORY: MaintenanceCategory = 'autre'

// ── Units ──

export const UNIT_TYPES = ['apartment', 'cellar', 'parking', 'commercial_unit', 'other'] as const satisfies readonly UnitType[]

// ── Allocation keys ──

export const ALLOCATION_KEY_TYPES = ['general', 'special'] as const satisfies readonly AllocationKeyType[]

// ── Registers ──

export const JUDICIAL_PROCEEDING_TYPES = ['debt_recovery', 'works_dispute', 'liability', 'other'] as const satisfies readonly JudicialProceedingType[]
export const JUDICIAL_PROCEEDING_STATUSES = ['ongoing', 'suspended', 'closed'] as const satisfies readonly JudicialProceedingStatus[]

/** Terminal judicial proceeding status. */
export const JUDICIAL_PROCEEDING_STATUS_CLOSED: JudicialProceedingStatus = 'closed'
export const GOVERNANCE_ROLES = ['syndic', 'council_president', 'council_member'] as const satisfies readonly GovernanceRole[]

/** Governance roles rendered with a distinct badge variant. */
export const GOVERNANCE_ROLE_SYNDIC: GovernanceRole = 'syndic'
export const GOVERNANCE_ROLE_COUNCIL_PRESIDENT: GovernanceRole = 'council_president'
export const ETAT_DATE_STATUSES = ['requested', 'generated', 'sent', 'cancelled'] as const satisfies readonly EtatDateStatus[]
export const ETAT_DATE_STATUS_REQUESTED: EtatDateStatus = 'requested'
export const ETAT_DATE_STATUS_GENERATED: EtatDateStatus = 'generated'
export const ETAT_DATE_STATUS_SENT: EtatDateStatus = 'sent'
export const ETAT_DATE_STATUS_CANCELLED: EtatDateStatus = 'cancelled'

// ── Calendar ──

export const CALENDAR_EVENT_CATEGORIES = ['reunion', 'visite', 'permanence', 'autre'] as const satisfies readonly CalendarEventCategory[]

// ── Urgency thresholds (days before due date) ──

/** Days before a reminder date to consider it high urgency. */
export const REMINDER_URGENCY_HIGH_DAYS = 14
/** Days before a reminder date to consider it medium urgency. */
export const REMINDER_URGENCY_MEDIUM_DAYS = 45

// ── UI filters ──

/** Pseudo-filter value meaning "no filter" in list views (not a domain status). */
export const ALL_FILTER = 'all' as const

// ── Defaults ──

export const DEFAULT_CLAIM_TYPE: ClaimType = 'current'
export const DEFAULT_CONTRACT_TYPE: ContractType = 'service'
export const DEFAULT_MEETING_TYPE: MeetingType = 'annual'
export const DEFAULT_UNIT_TYPE: UnitType = 'apartment'
export const DEFAULT_JUDICIAL_PROCEEDING_TYPE: JudicialProceedingType = 'other'
export const DEFAULT_JUDICIAL_PROCEEDING_STATUS: JudicialProceedingStatus = 'ongoing'
export const DEFAULT_GOVERNANCE_ROLE: GovernanceRole = 'syndic'

/** Meeting statuses that allow editing resolutions. */
export const RESOLUTION_EDITABLE_MEETING_STATUSES = ['draft', 'ongoing'] as const satisfies readonly MeetingStatus[]
/** Meeting statuses where the session (votes, attendance) is recorded. */
export const SESSION_ACTIVE_MEETING_STATUSES = ['ongoing', 'closed'] as const satisfies readonly MeetingStatus[]
/** Task statuses that still allow assignment changes. */
export const ASSIGNABLE_TASK_STATUSES = ['open', 'in_progress', 'awaiting_vendor', 'awaiting_validation'] as const satisfies readonly TaskStatus[]

// ── File formatting ──

export const BYTES_PER_KB = 1024
export const BYTES_PER_MB = 1024 * 1024
export const MS_PER_DAY = 1000 * 60 * 60 * 24
