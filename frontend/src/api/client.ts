import axios, { type AxiosInstance, type AxiosError } from 'axios'
import type { ApiError } from '@/types/domain'

// Augment axios config with a retry flag to prevent infinite loops
declare module 'axios' {
  interface InternalAxiosRequestConfig {
    _retry?: boolean
  }
}

// ── Instance principale ────────────────────────────────────────────
const API_TIMEOUT_MS = Number(import.meta.env.VITE_API_TIMEOUT_MS) || 5_000

export const api: AxiosInstance = axios.create({
  baseURL: '/api/v1',
  withCredentials: true,  // cookies httpOnly JWT
  headers: { 'Content-Type': 'application/json' },
  timeout: API_TIMEOUT_MS,
})

// ── Refresh state (prevents duplicate requests) ──────────────────
let isRefreshing = false
let refreshQueue: Array<(ok: boolean) => void> = []

function notifyQueue(ok: boolean) {
  refreshQueue.forEach(resolve => resolve(ok))
  refreshQueue = []
}

// Shared refresh flow used by both interceptor and restoreSession.
// This prevents refresh-token rotation races when multiple requests
// hit 401 at the same time.
export async function tryRefreshSession(): Promise<boolean> {
  if (isRefreshing) {
    return new Promise<boolean>(resolve => refreshQueue.push(resolve))
  }

  isRefreshing = true
  try {
    await api.post('/auth/refresh')
    notifyQueue(true)
    return true
  } catch {
    // Cross-tab race guard: another tab may have refreshed and rotated
    // the token just before this request failed with 401.
    // If /auth/me succeeds, session is still valid in this browser.
    try {
      await api.get('/auth/me')
      notifyQueue(true)
      return true
    } catch {
      notifyQueue(false)
      return false
    }
  } finally {
    isRefreshing = false
  }
}

// ── Response interceptor — transparent token refresh ─────────────────
// Auth probe requests — failures here are expected during session restoration
// and must NOT trigger a session-expired redirect.
// Identified by both URL and method to avoid blocking PATCH /auth/me.
function isAuthProbe(url: string | undefined, method: string | undefined): boolean {
  const u = url ?? ''
  const m = (method ?? '').toUpperCase()
  return (u === '/auth/me' && m === 'GET') || u === '/auth/refresh'
}

api.interceptors.response.use(
  response => response,
  async (error: AxiosError<ApiError>) => {
    const original = error.config
    if (!original) return Promise.reject(error)

    const status = error.response?.status

    // Any non-auth 401 should attempt a refresh once.
    // Backend error codes may vary by source (middleware/handler), but
    // the HTTP status is the reliable signal for an expired/invalid access token.
    const shouldRefresh = status === 401 && !original._retry

    if (shouldRefresh) {
      // Never attempt refresh or dispatch session-expired for auth probe requests
      // (called by restoreSession during initial session restoration)
      if (isAuthProbe(original.url, original.method)) {
        return Promise.reject(error)
      }

      const ok = await tryRefreshSession()
      if (!ok) {
        // Refresh failed → log out
        window.dispatchEvent(new CustomEvent('laruche:session-expired'))
        return Promise.reject(error)
      }

      original._retry = true
      return api.request(original)
    }

    return Promise.reject(error)
  }
)

// ── Helper d'extraction d'erreur ──────────────────────────────────
// Returns an i18n key to be translated by the caller.
// If the backend `message` field is already an i18n key path (starts with "errors."),
// it is used directly (specific validation/conflict code).
// Otherwise falls back to `errors.${data.error}` (generic HTTP error code).
export function extractApiError(error: unknown): string {
  if (axios.isAxiosError(error)) {
    const data = error.response?.data as ApiError | undefined
    if (data?.message?.startsWith('errors.')) return data.message
    if (data?.error) return `errors.${data.error}`
  }
  return 'errors.unknown'
}
