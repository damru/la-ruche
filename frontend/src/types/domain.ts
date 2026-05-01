// ── Domain types — exact mirror of Rust/PostgreSQL schema ─────────
// Any change here must stay consistent with the backend.

// ── Enums ─────────────────────────────────────────────────────────

export type Role = 'manager' | 'board_member' | 'co_owner' | 'tenant'

export type MeetingStatus = 'draft' | 'called' | 'held' | 'closed'
export type MeetingType = 'annual' | 'extraordinary'
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
export type ContractStatus = 'active' | 'terminated' | 'expired'
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
  email: string
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

export interface UnitShare {
  unit_id: string
  unit_number: string
  share_value: number
}

export interface User {
  id: string
  email: string
  last_name: string
  first_name: string
  active: boolean
  roles: Role[]
  created_at: string
  phone?: string
}

export interface Meeting {
  id: string
  meeting_type: MeetingType
  status: MeetingStatus
  notice_date?: string
  meeting_date: string
  location?: string
  created_at: string
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
  adopted: boolean
  votes_for: number
  votes_against: number
  abstentions: number
  shares_present: number
  total_shares: number
  explanation: string
}

export interface Budget {
  id: string
  fiscal_year: number
  status: BudgetStatus
  approval_meeting_id?: string
  created_at: string
  total_forecast?: number
  total_actual?: number
}

export interface BudgetLine {
  id: string
  budget_id: string
  label: string
  key_id: string
  key_name?: string
  budgeted_amount: number
  actual_amount?: number
}

export interface LevyCall {
  id: string
  budget_id: string
  unit_id: string
  unit_number?: string
  owner_name?: string
  amount: number
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
}

export interface Document {
  id: string
  original_name: string
  mime_type: string
  size_bytes: number
  visible_roles: Role[]
  linked_contract_names?: string[]
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
  created_by?: string
  /** True when a vendor token is active — the raw token UUID is never returned by list/get. */
  has_vendor_token?: boolean
  token_expires_at?: string
  closed_at?: string
  completion_cost?: number
  created_at: string
  /** True when at least one maintenance_log entry references this task. */
  is_in_logbook?: boolean
}

export interface Claim {
  id: string
  title: string
  description?: string
  claim_type: ClaimType
  declared_at: string
  published_at?: string
  status: ClaimStatus
  unit_id?: string
  transmitted_at?: string
  reimbursement_date?: string
  reimbursement_amount?: number
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
  /** Percentage of voters who selected this choice (0–100). */
  pct: number
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
    pct: number
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

export interface UnitChargesLevy {
  id: string
  fiscal_year: number
  amount: number
  due_date: string
  status: 'issued' | 'paid' | 'overdue'
  issued_at: string
  paid_at: string | null
  key_id: string | null
  key_name: string | null
  shares_snapshot: number | null
  shares_total_snapshot: number | null
}

export interface UnitChargesResponse {
  unit: { id: string; unit_number: string; unit_type: string }
  levies: UnitChargesLevy[]
  total_issued: number
  total_paid: number
  balance: number
}
