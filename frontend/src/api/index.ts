import { api } from './client'
import type {
  Meeting, Resolution, VoteResult,
  Budget, BudgetLine, LevyCall, WorksFund,
  VendorContract, ContractReminder,
  Task, Claim, MaintenanceEntry,
  Report, Consultation,
  Document, UploadUrlResponse, DownloadUrlResponse,
  Building, Unit, User,
  OnboardingStatus, BootstrapInput, BootstrapResponse,
  LoginResponse, BuildingStats,
} from '@/types/domain'

// ── Onboarding ────────────────────────────────────────────────────
export const onboardingApi = {
  status: () =>
    api.get<OnboardingStatus>('/onboarding/status'),
  bootstrap: (data: BootstrapInput) =>
    api.post<BootstrapResponse>('/onboarding/bootstrap', data),
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
  acceptInvite: (token: string, password: string, last_name: string, first_name: string) =>
    api.post('/auth/invite/accept', { token, password, last_name, first_name }),
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
}

// ── Users ─────────────────────────────────────────────────────────
export const usersApi = {
  list: () =>
    api.get<User[]>('/users'),
  get: (id: string) =>
    api.get<User>(`/users/${id}`),
  invite: (email: string, role: string, unit_id?: string) =>
    api.post('/users/invite', { email, role, unit_id }),
  updateRoles: (id: string, roles: string[]) =>
    api.put(`/users/${id}/roles`, { roles }),
  deactivate: (id: string) =>
    api.delete(`/users/${id}`),
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
  call: (id: string) =>
    api.post(`/meetings/${id}/call`),
  close: (id: string) =>
    api.post(`/meetings/${id}/close`),
  listResolutions: (meetingId: string) =>
    api.get<Resolution[]>(`/meetings/${meetingId}/resolutions`),
  createResolution: (meetingId: string, data: Partial<Resolution>) =>
    api.post<Resolution>(`/meetings/${meetingId}/resolutions`, data),
  updateResolution: (meetingId: string, rid: string, data: Partial<Resolution>) =>
    api.put<Resolution>(`/meetings/${meetingId}/resolutions/${rid}`, data),
  vote: (meetingId: string, rid: string, data: { unit_id: string; position: string; proxy_for?: string }) =>
    api.post(`/meetings/${meetingId}/resolutions/${rid}/vote`, data),
  results: (meetingId: string, rid: string) =>
    api.get<VoteResult>(`/meetings/${meetingId}/resolutions/${rid}/results`),
}

// ── Budget ────────────────────────────────────────────────────────
export const budgetApi = {
  list: () =>
    api.get<Budget[]>('/budgets'),
  get: (id: string) =>
    api.get<Budget>(`/budgets/${id}`),
  create: (data: Partial<Budget>) =>
    api.post<Budget>('/budgets', data),
  update: (id: string, data: Partial<Budget>) =>
    api.put<Budget>(`/budgets/${id}`, data),
  listLines: (id: string) =>
    api.get<BudgetLine[]>(`/budgets/${id}/lines`),
  createLine: (id: string, data: Partial<BudgetLine>) =>
    api.post<BudgetLine>(`/budgets/${id}/lines`, data),
  updateLine: (id: string, lid: string, data: Partial<BudgetLine>) =>
    api.put<BudgetLine>(`/budgets/${id}/lines/${lid}`, data),
  deleteLine: (id: string, lid: string) =>
    api.delete(`/budgets/${id}/lines/${lid}`),
  generateLevies: (id: string) =>
    api.post(`/budgets/${id}/levies/generate`),
  listLevies: (id: string) =>
    api.get<LevyCall[]>(`/budgets/${id}/levies`),
  markPaid: (levyId: string) =>
    api.put(`/levies/${levyId}/payment`),
  listWorksFund: () =>
    api.get<WorksFund[]>('/works-fund'),
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
  uploadToS3: async (file: File, signedUrl: string): Promise<void> => {
    await fetch(signedUrl, {
      method: 'PUT',
      body: file,
      headers: { 'Content-Type': file.type },
    })
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
    api.delete(`/contracts/${id}`),
  listReminders: (id: string) =>
    api.get<ContractReminder[]>(`/contracts/${id}/reminders`),
  acknowledgeReminder: (id: string, rid: string) =>
    api.put(`/contracts/${id}/reminders/${rid}/acknowledge`),
}

// ── Tasks ─────────────────────────────────────────────────────────
export const tasksApi = {
  list: () =>
    api.get<Task[]>('/tasks'),
  get: (id: string) =>
    api.get<Task>(`/tasks/${id}`),
  create: (data: Partial<Task>) =>
    api.post<Task>('/tasks', data),
  update: (id: string, data: Partial<Task>) =>
    api.put<Task>(`/tasks/${id}`, data),
  generateToken: (id: string) =>
    api.post<{ token: string; expires_at: string }>(`/tasks/${id}/vendor-token`),
  revokeToken: (id: string) =>
    api.delete(`/tasks/${id}/vendor-token`),
}

// ── Claims ────────────────────────────────────────────────────────
export const claimsApi = {
  list: () =>
    api.get<Claim[]>('/claims'),
  get: (id: string) =>
    api.get<Claim>(`/claims/${id}`),
  create: (data: Partial<Claim>) =>
    api.post<Claim>('/claims', data),
  update: (id: string, data: Partial<Claim>) =>
    api.put<Claim>(`/claims/${id}`, data),
}

// ── Maintenance Log ───────────────────────────────────────────────
export const maintenanceApi = {
  list: () =>
    api.get<MaintenanceEntry[]>('/logbook'),
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
  create: (data: Partial<Report>) =>
    api.post<Report>('/reports', data),
  updateStatus: (id: string, status: string) =>
    api.put(`/reports/${id}/status`, { status }),
}

// ── Consultations ─────────────────────────────────────────────────
export const consultationsApi = {
  list: () =>
    api.get<Consultation[]>('/consultations'),
  get: (id: string) =>
    api.get<Consultation>(`/consultations/${id}`),
  create: (data: Partial<Consultation>) =>
    api.post<Consultation>('/consultations', data),
  update: (id: string, data: Partial<Consultation>) =>
    api.put<Consultation>(`/consultations/${id}`, data),
  respond: (id: string, choice_id: string, comment?: string) =>
    api.post(`/consultations/${id}/respond`, { choice_id, comment }),
  results: (id: string) =>
    api.get(`/consultations/${id}/results`),
  close: (id: string) =>
    api.post(`/consultations/${id}/close`),
}
