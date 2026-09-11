// ── Domain types — exact mirror of Rust/PostgreSQL schema ─────────
// Any change here must stay consistent with the backend.

// ── Enums ─────────────────────────────────────────────────────────

export type Role = 'manager' | 'board_member' | 'co_owner' | 'tenant'

export type MeetingStatus = 'draft' | 'called' | 'ongoing' | 'closed'
export type MeetingType = 'annual' | 'extraordinary'
export type MeetingDocumentKind = 'notice' | 'attachment' | 'vote_by_mail_form'
export type MeetingAttendanceMode = 'present' | 'proxy' | 'correspondence' | 'absent'
export type VoteType = 'simple_majority' | 'absolute_majority' | 'double_majority' | 'unanimity'
export type ResolutionStatus = 'pending' | 'passed' | 'rejected' | 'withdrawn'
export type VotePosition = 'in_favor' | 'against' | 'abstain' | 'non_voter'

export type BudgetStatus = 'draft' | 'approved' | 'closed'
export type LevyStatus = 'issued' | 'paid' | 'overdue'

export type TaskStatus = 'open' | 'in_progress' | 'awaiting_vendor' | 'awaiting_validation' | 'closed' | 'cancelled'
export type TaskPriority = 'low' | 'normal' | 'high' | 'urgent'

export type ClaimType = 'current' | 'natural_disaster' | 'theft'
export type ClaimStatus = 'open' | 'in_progress' | 'closed'
export type ReportStatus = 'open' | 'in_progress' | 'closed'
export type PollStatus = 'draft' | 'open' | 'closed'

export type ContractType =
  | 'maintenance'
  | 'insurance'
  | 'service'
  | 'energy'
  | 'water'
  | 'cleaning'
  | 'green_spaces'
  | 'other'
export type ContractStatus = 'draft' | 'active' | 'terminated' | 'expired'
export type ReminderType = 'renewal' | 'payment_due' | 'expiration' | 'termination'
export type ReminderStatus = 'pending' | 'sent' | 'acknowledged'

export type PaymentFrequency = 'monthly' | 'quarterly' | 'biannual' | 'annual'
export type UnitType = 'apartment' | 'cellar' | 'parking' | 'commercial_unit' | 'other'

// ── Entities ──────────────────────────────────────────────────────

export interface Building {
  id: string
  name: string
  address_line1: string
  address_line2?: string
  postal_code: string
  city: string
  manager_siret?: string
  shares_total: number
  bylaws_date?: string
  created_at: string
}

export interface UnitOwner {
  id: string
  first_name: string
  last_name: string
  email?: string | null
}

export interface UnitOwnerRecord {
  id: string
  unit_id: string
  user_id: string | null
  display_name: string
  start_date: string
  end_date: string | null
  anonymized: boolean
}

export interface Unit {
  id: string
  building_id: string
  unit_number: string
  unit_type: UnitType
  block?: string
  staircase?: string
  floor?: string
  description?: string
  active: boolean
  current_owner?: UnitOwner
}

export type AllocationKeyType = 'general' | 'special'

export interface AllocationKey {
  id: string
  name: string
  description?: string
  key_type: AllocationKeyType
}

export interface AuditEntry {
  id: string
  building_id: string
  entity_type: string
  entity_id: string
  operation: string
  changed_by?: string
  changed_at: string
  old_values?: Record<string, unknown>
  new_values?: Record<string, unknown>
}

export interface UnitShare {
  unit_id: string
  unit_number: string
  share_value: number
}

export interface User {
  id: string
  email?: string | null
  last_name: string
  first_name: string
  active: boolean
  roles: Role[]
  created_at?: string
  phone?: string | null
  status?: 'active' | 'pending_invite' | 'no_access' | 'archived'
  expires_at?: string
  invite_token_id?: string
  has_access?: boolean
}

export interface MeetingScrutineer {
  name: string
  user_id?: string | null
}

export interface Meeting {
  id: string
  meeting_type: MeetingType
  status: MeetingStatus
  notice_date?: string
  meeting_date: string
  location?: string
  agenda?: string
  minutes_document_id?: string
  created_at: string
  resolution_count: number
  passed_count: number
  president_name?: string | null
  president_user_id?: string | null
  secretary_name?: string | null
  secretary_user_id?: string | null
  scrutineers: MeetingScrutineer[]
  minutes_notified_at?: string | null
  convocation_sent_at?: string | null
  convocation_channel?: 'lrar' | 'hand_delivery' | 'email' | null
  minutes_president_signed_at?: string | null
  minutes_secretary_signed_at?: string | null
  minutes_scrutineers_signed?: Array<{ name: string; signed_at: string }>
}

export type ConvocationChannel = 'lrar' | 'hand_delivery' | 'email'

export interface ConvocationChecklistItem {
  key: string
  satisfied: boolean
  severity: 'Blocking' | 'Advisory'
}

export interface ConvocationChecklist {
  items: ConvocationChecklistItem[]
  ready_to_send: boolean
}

export interface MeetingDocument extends Document {
  kind: MeetingDocumentKind
}

export interface MeetingAttendanceEntry {
  unit_id: string
  unit_number: string
  owner_name?: string
  mode: MeetingAttendanceMode
  representative_name?: string
  shares_represented: number
}

export interface MeetingAttendance {
  entries: MeetingAttendanceEntry[]
  shares_present: number
  total_shares: number
  present_units: number
}

export interface Resolution {
  id: string
  meeting_id: string
  number: number
  title: string
  description?: string
  vote_type: VoteType
  status: ResolutionStatus
}

export interface VoteResult {
  passed: boolean
  for_votes: number
  against_votes: number
  abstentions: number
  shares_present: number
  shares_total: number
  shares_counted: number
  threshold: number
  eligible_for_second_vote: 'art_25_1' | 'art_26_1' | null
}

export interface UnitVote {
  unit_id: string
  position: 'in_favor' | 'against' | 'abstain'
}

export interface Budget {
  id: string
  fiscal_year: number
  status: BudgetStatus
  approval_meeting_id?: string
  /** Set on a rectificatif: the budget it amends (Pattern B, ADR-009). */
  parent_budget_id?: string
  created_at: string
  total_forecast?: number
  total_actual?: number
}

export interface AccountingCode {
  code: string
  label: string
  category: string
}

export interface BudgetLine {
  id: string
  budget_id: string
  label: string
  key_id: string
  key_name?: string
  budgeted_amount: number
  actual_amount?: number
  account_code?: string
  account_code_label?: string
  account_code_category?: string
  /** Always present: the date this version took effect. */
  valid_from: string
  /** Set once the version is closed by a rectificatif; inclusive bound. */
  valid_to?: string
  /** NULL on a root budget's lines, 1..N on a rectificatif's. */
  rectificatif_version?: number
  /** Optimistic-locking token — must be echoed back on update. */
  version: number
}

export interface LevyCall {
  id: string
  budget_id: string
  unit_id: string
  call_label: string
  owner_id?: string
  unit_number?: string
  owner_name?: string
  amount: number
  charge_amount?: number
  works_fund_amount?: number
  due_date: string
  status: LevyStatus
  issued_at: string
  paid_at?: string
}

export interface WorksFund {
  id: string
  fiscal_year: number
  levied_amount: number
  available_amount: number
  contribution_rate_pct: number
  minimum_legal_rate_pct: number
  is_rate_compliant: boolean
}

export interface Document {
  id: string
  original_name: string
  mime_type: string
  size_bytes: number
  visible_roles: Role[]
  linked_contract_names?: string[]
  linked_report_titles?: string[]
  linked_claim_titles?: string[]
  linked_report_count?: number
  created_at: string
}

export interface VendorContract {
  id: string
  vendor_name: string
  vendor_email?: string
  vendor_phone?: string
  subject: string
  contract_type: ContractType
  start_date: string
  end_date?: string
  amount?: number
  payment_frequency?: PaymentFrequency
  auto_renewal: boolean
  notice_days?: number
  status: ContractStatus
  predecessor_id?: string | null
  document_count?: number
  created_at: string
}

export interface ContractReminder {
  id: string
  contract_id: string
  reminder_type: ReminderType
  reminder_date: string
  target_date?: string
  status: ReminderStatus
}

export interface Task {
  id: string
  title: string
  description?: string
  status: TaskStatus
  priority: TaskPriority
  contract_id?: string
  contractor_id?: string
  contractor_name?: string
  assigned_to?: string
  created_by: string
  /** True when a vendor token is active — the raw token UUID is never returned by list/get. */
  has_vendor_token?: boolean
  token_expires_at?: string
  closed_at?: string
  completion_cost?: number
  created_at: string
  updated_at: string
  document_count: number
  /** True when at least one maintenance_log entry references this task. */
  is_in_logbook?: boolean
}

export interface Claim {
  id: string
  created_by: string
  title: string
  description?: string
  claim_type: ClaimType
  declared_at: string
  published_at?: string
  status: ClaimStatus
  unit_id?: string
  transmitted_at?: string
  reimbursement_date?: string
  reimbursement_amount?: number | string
  legal_deadline_at: string
  days_remaining?: number
  is_deadline_approaching: boolean
  is_deadline_missed: boolean
  is_transmitted_late: boolean
  document_count: number
  created_at: string
  updated_at: string
}

export interface MaintenanceEntry {
  id: string
  title: string
  description?: string
  intervention_date: string
  cost?: number
  vendor_name?: string
  contract_id?: string
  task_id?: string
  source: 'manual' | 'automatic'
  created_at: string
}

export interface LogbookContractSummary {
  id: string
  vendor_name: string
  subject: string
  contract_type: ContractType
  end_date?: string
  status: ContractStatus
}

export interface LogbookSummary {
  building: Pick<Building, 'name' | 'address_line1' | 'address_line2' | 'postal_code' | 'city'>
  current_manager?: Pick<User, 'first_name' | 'last_name' | 'email' | 'phone'>
  insurance_contracts: LogbookContractSummary[]
  maintenance_contracts: LogbookContractSummary[]
}

export interface Report {
  id: string
  unit_id?: string
  author_id: string
  author_first_name: string
  author_last_name: string
  title: string
  description?: string
  status: ReportStatus
  task_id?: string
  task_status?: TaskStatus
  document_count: number
  created_at: string
  updated_at: string
}

export interface Consultation {
  id: string
  title: string
  description?: string
  status: PollStatus
  visible_roles: Role[]
  closed_at?: string
  created_at: string
  updated_at: string
  choices: ConsultationChoice[]
  /** IDs of the choices the current user has already selected (empty = not voted yet). */
  my_choice_ids: string[]
  /** Total number of distinct voters. */
  total_responses: number
}

export interface ConsultationChoice {
  id: string
  label: string
  order_index: number
  /** Number of voters who selected this choice (from list endpoint). */
  votes: number
}

// ── Auth ──────────────────────────────────────────────────────────

export interface AuthUser {
  user_id: string
  building_id: string
  roles: Role[]
  last_name: string
  first_name: string
  email: string
  phone?: string
}

// ── Generic API responses ─────────────────────────────────────────

export interface ApiError {
  error: string
  message: string
}

export interface UploadUrlResponse {
  url: string
  key: string
  bucket: string
}

export interface DownloadUrlResponse {
  url: string
}

// ── Onboarding ───────────────────────────────────────────────────

export interface OnboardingStatus {
  needs_onboarding: boolean
}

export interface RuntimeStatus {
  demo_mode: boolean
}

export interface BootstrapInput {
  email: string
  last_name: string
  first_name: string
  password: string
  building_name: string
  address_line1: string
  postal_code: string
  city: string
  shares_total: number
}

export interface BootstrapResponse {
  user_id: string
  building_id: string
  roles: Role[]
  last_name: string
  first_name: string
}

export interface LoginResponse {
  user_id: string
  building_id: string
  roles: Role[]
  last_name: string
  first_name: string
  email?: string
  phone?: string
}

export interface BuildingStats {
  name: string
  nb_lots: number
  nb_users: number
  nb_tenants: number
  shares_total: number
  works_fund_available: string
  works_fund_pct: number | null
}

// ── Registry ─────────────────────────────────────────────────────

export interface RegistryKey {
  id: string
  name: string
  description?: string
  shares_sum: number
  shares: UnitShare[]
}

export interface RegistryResponse {
  building: {
    id: string
    name: string
    address_line1: string
    postal_code: string
    city: string
    shares_total: number
  }
  units: {
    id: string
    unit_number: string
    unit_type: UnitType
    block?: string
    staircase?: string
    floor?: string
    active: boolean
  }[]
  keys: RegistryKey[]
}

export interface ConsultationResults {
  total: number
  choices: {
    id: string
    label: string
    votes: number
  }[]
}

/// Task visible to an external vendor (token-based, no auth).
export interface VendorTask {
  id: string
  title: string
  description?: string
  status: string
  token_expires_at?: string
  building_name: string
  building_address: string
  manager_name: string
  manager_email: string
  manager_phone?: string
}

/// Contractor from the building's referential.
export interface Contractor {
  id: string
  building_id: string
  name: string
  email?: string
  phone?: string
  /// Professional domains (multi-value).
  domains: string[]
  notes?: string
  created_at: string
  updated_at: string
}

/// Document metadata sent in a vendor submission.
export interface VendorDocument {
  key: string
  original_name: string
  mime_type: string
  size_bytes: number
}

export interface UnitChargesLevyBreakdownItem {
  budget_line_id: string | null
  budget_line_label: string | null
  key_id: string | null
  key_name: string | null
  shares_snapshot: number | null
  shares_total_snapshot: number | null
  amount: number
  is_works_fund: boolean
}

export interface UnitChargesLevy {
  id: string
  fiscal_year: number
  call_label: string
  due_date: string
  status: 'issued' | 'paid' | 'overdue'
  issued_at: string
  paid_at: string | null
  total_amount: number
  charge_amount: number
  works_fund_amount: number
  breakdown: UnitChargesLevyBreakdownItem[]
}

export interface UnitChargesResponse {
  unit: { id: string; unit_number: string; unit_type: string }
  levies: UnitChargesLevy[]
  total_issued: number
  total_paid: number
  balance: number
  pending_transfer: boolean
}

// ── Judicial proceedings register (EP-002-S10) ───────────────────

export type JudicialProceedingType = 'debt_recovery' | 'works_dispute' | 'liability' | 'other'
export type JudicialProceedingStatus = 'ongoing' | 'suspended' | 'closed'

export interface JudicialProceeding {
  id: string
  building_id: string
  proceeding_type: JudicialProceedingType
  status: JudicialProceedingStatus
  subject: string
  claimant: string
  defendant: string
  jurisdiction: string | null
  case_reference: string | null
  opened_at: string
  closed_at: string | null
  outcome: string | null
  created_by: string
  created_at: string
  updated_at: string
}

// ── EDD coherence check (EP-001-S09) ──────────────────────────────

export interface EddReference {
  id: string
  label: string
  edd_date: string
  document_id?: string | null
  created_at: string
}

export interface EddReferenceShareInput {
  unit_id: string
  allocation_key_id: string
  share_value: number
}

export type EddDiscrepancy =
  | { kind: 'missing_in_edd'; unit_id: string; allocation_key_id: string }
  | { kind: 'missing_in_config'; unit_id: string; allocation_key_id: string }
  | { kind: 'value_mismatch'; unit_id: string; allocation_key_id: string; expected: number; actual: number }
  | { kind: 'total_mismatch'; allocation_key_id: string; expected: number; actual: number }

export interface EddCheckResponse {
  reference: EddReference
  discrepancies: EddDiscrepancy[]
}

// ── Governance register (EP-002-S11) ─────────────────────────────

export type GovernanceRole = 'syndic' | 'council_president' | 'council_member'

export interface GovernanceMandate {
  id: string
  building_id: string
  role: GovernanceRole
  holder_name: string
  user_id: string | null
  start_date: string
  end_date: string | null
  notes: string | null
  created_by: string
  created_at: string
  updated_at: string
}
