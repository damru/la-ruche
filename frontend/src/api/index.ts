import { api } from './client'
import axios from 'axios'
import type {
  Meeting, MeetingAttendance, MeetingDocument, Resolution, VoteResult, UnitVote,
  Budget, BudgetLine, LevyCall, WorksFund,
  VendorContract, ContractReminder,
    Task, Claim, MaintenanceEntry,
  LogbookSummary,
  Report, Consultation, ConsultationResults, VendorTask, VendorDocument,
  Document, UploadUrlResponse, DownloadUrlResponse,
  Building, Unit, User, AllocationKey, UnitShare,
  OnboardingStatus, RuntimeStatus, BootstrapInput, BootstrapResponse,
  LoginResponse, BuildingStats, RegistryResponse, UnitChargesResponse,
  Contractor,
} from '@/types/domain'

// Public axios instance — no JWT, no interceptors (vendor endpoints)
const publicApi = axios.create({
  baseURL: '/',
  headers: { 'Content-Type': 'application/json' },
  timeout: 10_000,
})

// ── Onboarding ────────────────────────────────────────────────────
export const onboardingApi = {
  status: () =>
    api.get<OnboardingStatus>('/onboarding/status'),
  bootstrap: (data: BootstrapInput) =>
    api.post<BootstrapResponse>('/onboarding/bootstrap', data),
}

export const runtimeApi = {
  status: () =>
    api.get<RuntimeStatus>('/runtime'),
}

// ── Auth ──────────────────────────────────────────────────────────
export const authApi = {
  login: (email: string, password: string) =>
    api.post<LoginResponse>('/auth/login', { email, password }),
  logout: () =>
    api.post('/auth/logout'),
  refresh: () =>
    api.post('/auth/refresh'),
  requestReset: (email: string) =>
    api.post('/auth/reset-password/request', { email }),
  confirmReset: (token: string, password: string) =>
    api.post('/auth/reset-password/confirm', { token, password }),
  me: () =>
    api.get<LoginResponse>('/auth/me'),
  updateProfile: (data: { phone?: string | null }) =>
    api.patch<LoginResponse>('/auth/me', data),
  acceptInvite: (token: string, password: string, last_name: string, first_name: string) =>
    api.post('/auth/invite/accept', { token, password, last_name, first_name }),
  validateInvite: (token: string) =>
    api.get<{ email: string; role: string | null; unit_id: string | null; expires_at: string }>(`/auth/invite/validate?token=${encodeURIComponent(token)}`),
  passkeyRegisterStart: () =>
    api.post('/auth/passkey/register/start'),
  passkeyRegisterFinish: (credential: unknown) =>
    api.post('/auth/passkey/register/finish', { credential }),
  passkeyLoginStart: (email: string) =>
    api.post('/auth/passkey/login/start', { email }),
  passkeyLoginFinish: (credential: unknown) =>
    api.post('/auth/passkey/login/finish', { credential }),
}

// ── Building ──────────────────────────────────────────────────────
export const buildingApi = {
  get: () =>
    api.get<Building>('/building'),
  update: (data: Partial<Building>) =>
    api.put<Building>('/building', data),
  stats: () =>
    api.get<BuildingStats>('/building/stats'),
  registry: () =>
    api.get<RegistryResponse>('/building/registry'),
}

// ── Allocation keys ───────────────────────────────────────────────
export const allocationKeysApi = {
  list: () =>
    api.get<AllocationKey[]>('/building/keys'),
  create: (data: { name: string; description?: string }) =>
    api.post<AllocationKey>('/building/keys', data),
  update: (id: string, data: { name: string; description?: string }) =>
    api.put<AllocationKey>(`/building/keys/${id}`, data),
  delete: (id: string) =>
    api.delete(`/building/keys/${id}`),
  listShares: (id: string) =>
    api.get<{ shares: UnitShare[]; shares_sum: number; shares_total: number }>(`/building/keys/${id}/shares`),
  setShares: (id: string, shares: { unit_id: string; share_value: number }[]) =>
    api.put(`/building/keys/${id}/shares`, { shares }),
  balance: (id: string) =>
    api.get<{ shares_sum: number; shares_total: number; balanced: boolean; delta: number }>(`/building/keys/${id}/balance`),
}

// ── Units ─────────────────────────────────────────────────────────
export const unitsApi = {
  list: () =>
    api.get<Unit[]>('/units'),
  get: (id: string) =>
    api.get<Unit>(`/units/${id}`),
  create: (data: Partial<Unit>) =>
    api.post<Unit>('/units', data),
  update: (id: string, data: Partial<Unit>) =>
    api.put<Unit>(`/units/${id}`, data),
  archive: (id: string) =>
    api.delete(`/units/${id}`),
  owners: (id: string) =>
    api.get(`/units/${id}/owners`),
  transfer: (id: string, data: { user_id: string; start_date: string }) =>
    api.post(`/units/${id}/transfer`, data),
  charges: (id: string) =>
    api.get<UnitChargesResponse>(`/units/${id}/charges`),
}

// ── Users ─────────────────────────────────────────────────────────
export const usersApi = {
  list: () =>
    api.get<User[]>('/users'),
  get: (id: string) =>
    api.get<User>(`/users/${id}`),
  invite: (email: string, role: string, unit_id?: string) =>
    api.post<{ token: string; email: string; role: string; expires_at: string; invite_link: string }>('/users/invite', { email, role, unit_id }),
  revokeInvite: (tokenId: string) =>
    api.delete(`/users/invites/${tokenId}`),
  updateRoles: (id: string, roles: string[]) =>
    api.put(`/users/${id}/roles`, { roles }),
  deactivate: (id: string) =>
    api.delete(`/users/${id}`),
  anonymize: (id: string, confirm_name: string) =>
    api.post(`/users/${id}/anonymize`, { confirm_name }),
}

// ── Meetings ──────────────────────────────────────────────────────
export const meetingsApi = {
  list: () =>
    api.get<Meeting[]>('/meetings'),
  get: (id: string) =>
    api.get<Meeting>(`/meetings/${id}`),
  create: (data: Partial<Meeting>) =>
    api.post<Meeting>('/meetings', data),
  update: (id: string, data: Partial<Meeting>) =>
    api.put<Meeting>(`/meetings/${id}`, data),
  // EP-003-S09 — channel et date d'envoi requis pour preuve d'envoi
  call: (id: string, data: { channel: 'lrar' | 'hand_delivery' | 'email'; sent_at?: string }) =>
    api.post(`/meetings/${id}/call`, data),
  getConvocationChecklist: (id: string) =>
    api.get<import('@/types/domain').ConvocationChecklist>(`/meetings/${id}/convocation-checklist`),
  start: (id: string, data: {
    president_name: string
    president_user_id?: string | null
    secretary_name: string
    secretary_user_id?: string | null
    scrutineers: Array<{ name: string; user_id?: string | null }>
  }) =>
    api.post(`/meetings/${id}/start`, data),
  close: (id: string) =>
    api.post(`/meetings/${id}/close`),
  patchResolution: (meetingId: string, rid: string, data: { description?: string }) =>
    api.patch<Resolution>(`/meetings/${meetingId}/resolutions/${rid}`, data),
  listDocuments: (id: string) =>
    api.get<MeetingDocument[]>(`/meetings/${id}/documents`),
  attachDocument: (id: string, data: { document_id: string; kind: 'notice' | 'attachment' }) =>
    api.post(`/meetings/${id}/documents`, data),
  detachDocument: (id: string, documentId: string) =>
    api.delete(`/meetings/${id}/documents/${documentId}`),
  attendance: (id: string) =>
    api.get<MeetingAttendance>(`/meetings/${id}/attendance`),
  saveAttendance: (id: string, entries: MeetingAttendance['entries']) =>
    api.put(`/meetings/${id}/attendance`, {
      entries: entries.map(entry => ({
        unit_id: entry.unit_id,
        mode: entry.mode,
        representative_name: entry.representative_name || undefined,
      })),
    }),
  listResolutions: (meetingId: string) =>
    api.get<Resolution[]>(`/meetings/${meetingId}/resolutions`),
  createResolution: (meetingId: string, data: Partial<Resolution>) =>
    api.post<Resolution>(`/meetings/${meetingId}/resolutions`, data),
  updateResolution: (meetingId: string, rid: string, data: Partial<Resolution>) =>
    api.put<Resolution>(`/meetings/${meetingId}/resolutions/${rid}`, data),
  vote: (meetingId: string, rid: string, data: { unit_id: string; position: string; proxy_for?: string }) =>
    api.post(`/meetings/${meetingId}/resolutions/${rid}/vote`, data),
  listVotes: (meetingId: string, rid: string) =>
    api.get<UnitVote[]>(`/meetings/${meetingId}/resolutions/${rid}/votes`),
  results: (meetingId: string, rid: string) =>
    api.get<VoteResult>(`/meetings/${meetingId}/resolutions/${rid}/results`),
  finalizeResolution: (meetingId: string, rid: string) =>
    api.post<{ finalized: boolean; result?: VoteResult; eligible_for_second_vote: 'art_25_1' | 'art_26_1' | null }>(
      `/meetings/${meetingId}/resolutions/${rid}/finalize`,
    ),
  // EP-013-S08 — Draft PV (structured markdown)
  getMinutesDraft: (id: string) =>
    api.get<{ markdown: string }>(`/meetings/${id}/minutes/draft`),
  // EP-003-S05 — Attaches a signed PV already uploaded (pre-signed URL EP-005)
  attachMinutes: (id: string, document_id: string) =>
    api.post(`/meetings/${id}/minutes`, { document_id }),
  // EP-003-S06 — Marks the PV as notified to absent co-owners (art. 42 loi 1965 deadline)
  notifyMinutes: (id: string) =>
    api.post(`/meetings/${id}/notify-minutes`),
  // EP-003-S10 — Records bureau de séance signatures on the PV
  signMinutes: (id: string, data: {
    president_signed: boolean
    secretary_signed: boolean
    scrutineers_signed: string[]
  }) =>
    api.post(`/meetings/${id}/minutes/signatures`, data),
}

// ── Budget ────────────────────────────────────────────────────────
export const budgetApi = {
  list: () =>
    api.get<Budget[]>('/budgets'),
  get: (id: string) =>
    api.get<Budget>(`/budgets/${id}`),
  create: (data: { fiscal_year: number }) =>
    api.post<Budget>('/budgets', data),
  close: (id: string) =>
    api.post(`/budgets/${id}/close`),
  listLines: (id: string) =>
    api.get<BudgetLine[]>(`/budgets/${id}/lines`),
  createLine: (id: string, data: { label: string; key_id: string; budgeted_amount: number }) =>
    api.post<BudgetLine>(`/budgets/${id}/lines`, data),
  deleteLine: (id: string, lid: string) =>
    api.delete(`/budgets/${id}/lines/${lid}`),
  generateLevies: (id: string, data: { due_date: string; include_works_fund: boolean; call_label?: string }) =>
    api.post<{ generated: number; total_amount: number; already_called: number; remaining_amount: number; due_date: string; call_label?: string }>(`/budgets/${id}/levies/generate`, data),
  listLevies: (id: string) =>
    api.get<LevyCall[]>(`/budgets/${id}/levies`),
  markPaid: (levyCallId: string, unitId: string) =>
    api.put(`/levies/${levyCallId}/units/${unitId}/payment`),
  getWorksFund: (fiscalYear: number) =>
    api.get<WorksFund>(`/works-fund/${fiscalYear}`),
  updateWorksFund: (fiscalYear: number, data: { contribution_rate_pct: number }) =>
    api.put<WorksFund>(`/works-fund/${fiscalYear}`, data),
}

// ── Documents ─────────────────────────────────────────────────────
export const documentsApi = {
  list: () =>
    api.get<Document[]>('/documents'),
  getUploadUrl: (extension: string, content_type: string, size_bytes: number) =>
    api.post<UploadUrlResponse>('/documents/upload-url', { extension, content_type, size_bytes }),
  confirmUpload: (data: { key: string; bucket: string; original_name: string; mime_type: string; size_bytes: number; visible_roles: string[] }) =>
    api.post<Document>('/documents', data),
  getDownloadUrl: (id: string) =>
    api.get<DownloadUrlResponse>(`/documents/${id}/download-url`),
  delete: (id: string) =>
    api.delete(`/documents/${id}`),
  uploadToS3: async (file: File, signedUrl: string, contentType?: string): Promise<void> => {
    const response = await fetch(signedUrl, {
      method: 'PUT',
      body: file,
      headers: { 'Content-Type': contentType ?? file.type },
    })

    if (!response.ok) {
      throw new Error(`S3 upload failed (${response.status})`)
    }
  },
}

// ── Contracts ─────────────────────────────────────────────────────
export const contractsApi = {
  list: () =>
    api.get<VendorContract[]>('/contracts'),
  get: (id: string) =>
    api.get<VendorContract>(`/contracts/${id}`),
  create: (data: Partial<VendorContract>) =>
    api.post<VendorContract>('/contracts', data),
  update: (id: string, data: Partial<VendorContract>) =>
    api.put<VendorContract>(`/contracts/${id}`, data),
  terminate: (id: string) =>
    api.put(`/contracts/${id}/terminate`),
  delete: (id: string) =>
    api.delete(`/contracts/${id}`),
  listAllReminders: () =>
    api.get<ContractReminder[]>('/contracts/reminders'),
  listReminders: (id: string) =>
    api.get<ContractReminder[]>(`/contracts/${id}/reminders`),
  acknowledgeReminder: (id: string, rid: string) =>
    api.put(`/contracts/${id}/reminders/${rid}/acknowledge`),
  listDocuments: (id: string) =>
    api.get<Document[]>(`/contracts/${id}/documents`),
  attachDocument: (id: string, document_id: string) =>
    api.post(`/contracts/${id}/documents`, { document_id }),
  detachDocument: (id: string, documentId: string) =>
    api.delete(`/contracts/${id}/documents/${documentId}`),
}

// ── Tasks ─────────────────────────────────────────────────────────
export const tasksApi = {
  list: () =>
    api.get<Task[]>('/tasks'),
  get: (id: string) =>
    api.get<Task>(`/tasks/${id}`),
  create: (data: { title: string; description?: string; priority?: string; contract_id?: string; contractor_id?: string | null; contractor_name?: string | null; assigned_to?: string }) =>
    api.post<Task>('/tasks', data),
  update: (id: string, data: { title?: string; description?: string | null; priority?: string; status?: string; contractor_id?: string | null; contractor_name?: string | null; completion_cost?: number | null }) =>
    api.put<Task>(`/tasks/${id}`, data),
  delete: (id: string) =>
    api.delete(`/tasks/${id}`),
  listDocuments: (id: string) =>
    api.get<Document[]>(`/tasks/${id}/documents`),
  attachDocument: (id: string, document_id: string) =>
    api.post(`/tasks/${id}/documents`, { document_id }),
  detachDocument: (id: string, documentId: string) =>
    api.delete(`/tasks/${id}/documents/${documentId}`),
  generateToken: (id: string) =>
    api.post<{ token: string; expires_at: string; url: string }>(`/tasks/${id}/vendor-token`),
  getVendorTokenUrl: (id: string) =>
    api.get<{ url: string; expires_at: string }>(`/tasks/${id}/vendor-token`),
  revokeToken: (id: string) =>
    api.delete(`/tasks/${id}/vendor-token`),
}

// ── Contractors referential ───────────────────────────────────────
export const contractorsApi = {
  list: (q?: string) =>
    api.get<Contractor[]>('/contractors', { params: q ? { q } : undefined }),
  get: (id: string) =>
    api.get<Contractor>(`/contractors/${id}`),
  create: (data: { name: string; email?: string; phone?: string; domains?: string[]; notes?: string }) =>
    api.post<Contractor>('/contractors', data),
  update: (id: string, data: { name?: string; email?: string; phone?: string; domains?: string[]; notes?: string }) =>
    api.put<Contractor>(`/contractors/${id}`, data),
  delete: (id: string) =>
    api.delete(`/contractors/${id}`),
}

// ── Claims ────────────────────────────────────────────────────────
export const claimsApi = {
  list: () =>
    api.get<Claim[]>('/claims'),
  get: (id: string) =>
    api.get<Claim>(`/claims/${id}`),
  create: (data: { title: string; description?: string; claim_type: string; declared_at: string; published_at?: string; unit_id?: string }) =>
    api.post<Claim>('/claims', data),
  update: (id: string, data: { title: string; description?: string | null; claim_type: string; declared_at: string; published_at?: string | null; status: string; unit_id?: string | null; transmitted_at?: string | null; reimbursement_date?: string | null; reimbursement_amount?: string | null }) =>
    api.put<Claim>(`/claims/${id}`, data),
  delete: (id: string) =>
    api.delete(`/claims/${id}`),
  listDocuments: (id: string) =>
    api.get<Document[]>(`/claims/${id}/documents`),
  attachDocument: (id: string, document_id: string) =>
    api.post(`/claims/${id}/documents`, { document_id }),
  detachDocument: (id: string, document_id: string) =>
    api.delete(`/claims/${id}/documents/${document_id}`),
}

// ── Maintenance Log ───────────────────────────────────────────────
export const maintenanceApi = {
  list: () =>
    api.get<MaintenanceEntry[]>('/logbook'),
  summary: () =>
    api.get<LogbookSummary>('/logbook/summary'),
  get: (id: string) =>
    api.get<MaintenanceEntry>(`/logbook/${id}`),
  create: (data: Partial<MaintenanceEntry>) =>
    api.post<MaintenanceEntry>('/logbook', data),
  update: (id: string, data: Partial<MaintenanceEntry>) =>
    api.put<MaintenanceEntry>(`/logbook/${id}`, data),
  delete: (id: string) =>
    api.delete(`/logbook/${id}`),
}

// ── Reports ───────────────────────────────────────────────────────
export const reportsApi = {
  list: () =>
    api.get<Report[]>('/reports'),
  get: (id: string) =>
    api.get<Report>(`/reports/${id}`),
  create: (data: { title: string; description?: string; unit_id?: string }) =>
    api.post<Report>('/reports', data),
  update: (id: string, data: { title: string; description: string | null; unit_id: string | null }) =>
    api.put(`/reports/${id}`, data),
  delete: (id: string) =>
    api.delete(`/reports/${id}`),
  updateStatus: (id: string, status: string) =>
    api.put(`/reports/${id}/status`, { status }),
  convertToTask: (id: string, data?: { priority?: string }) =>
    api.post<{ task_id: string }>(`/reports/${id}/task`, data),
  listDocuments: (id: string) =>
    api.get<Document[]>(`/reports/${id}/documents`),
  attachDocument: (id: string, document_id: string) =>
    api.post(`/reports/${id}/documents`, { document_id }),
  detachDocument: (id: string, document_id: string) =>
    api.delete(`/reports/${id}/documents/${document_id}`),
}

// ── Consultations ─────────────────────────────────────────────────
export const consultationsApi = {
  list: () =>
    api.get<{ consultations: Consultation[] }>('/consultations'),
  get: (id: string) =>
    api.get<{ consultation: Consultation }>(`/consultations/${id}`),
  create: (data: { title: string; description?: string; visible_to_roles: string[]; choices: string[]; closed_at?: string }) =>
    api.post<Consultation>('/consultations', data),
  /** Opens a draft consultation (transitions draft → open). */
  openDraft: (id: string) =>
    api.put(`/consultations/${id}`),
  respond: (id: string, choice_ids: string[], comment?: string) =>
    api.post(`/consultations/${id}/respond`, { choice_ids, comment }),
  results: (id: string) =>
    api.get<{ results: ConsultationResults }>(`/consultations/${id}/results`),
  close: (id: string) =>
    api.post(`/consultations/${id}/close`),
  delete: (id: string) =>
    api.delete(`/consultations/${id}`),
}

// ── Contractor (public, tokenized — no JWT) ───────────────────────
// Vendor API routes now live under /api/v1/vendor/ so the SPA can serve
// /vendor/:token as a real page without backend interference.
export const vendorApi = {
  getTask: (token: string) =>
    publicApi.get<VendorTask>(`/api/v1/vendor/${token}`),
  getUploadUrl: (token: string, extension: string, content_type: string) =>
    publicApi.post<{ url: string; key: string }>(`/api/v1/vendor/${token}/upload-url`, { extension, content_type }),
  submit: (token: string, data: { comment?: string; cost?: number; documents: VendorDocument[] }) =>
    publicApi.post(`/api/v1/vendor/${token}/submission`, data),
  uploadToStaging: async (file: File, signedUrl: string): Promise<void> => {
    // Keep the upload header aligned with the content type used to generate the
    // presigned URL. Some browser-provided File objects have an empty `type`.
    const contentType = file.type || 'application/octet-stream'

    const response = await fetch(signedUrl, {
      method: 'PUT',
      body: file,
      headers: { 'Content-Type': contentType },
    })
    if (!response.ok) {
      throw new Error(`Staging upload failed (${response.status})`)
    }
  },
}

// ── Admin ─────────────────────────────────────────────────────────
export const adminApi = {
  resetDemo: () =>
    api.post('/admin/reset-demo'),
  resetData: () =>
    api.post('/admin/reset-data'),
}
