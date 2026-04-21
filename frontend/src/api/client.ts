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

// ── Response interceptor — transparent token refresh ─────────────────
// Auth probe URLs — failures here are expected during session restoration
// and must NOT trigger a session-expired redirect.
const AUTH_PROBE_URLS = new Set(['/auth/me', '/auth/refresh'])

api.interceptors.response.use(
  response => response,
  async (error: AxiosError<ApiError>) => {
    const original = error.config
    if (!original) return Promise.reject(error)

    const status = error.response?.status
    const errorCode = error.response?.data?.error

    // 401 TOKEN_INVALID  → token présent mais expiré
    // 401 UNAUTHORIZED   → cookie supprimé par le browser après Max-Age
    // Dans les deux cas, tenter un refresh (sauf sur les routes auth elles-mêmes,
    // et sauf si la requête a déjà été retentée après un refresh)
    const shouldRefresh =
      status === 401 &&
      (errorCode === 'TOKEN_INVALID' || errorCode === 'UNAUTHORIZED') &&
      !original._retry

    if (shouldRefresh) {
      // Never attempt refresh or dispatch session-expired for auth probe requests
      // (called by restoreSession during initial session restoration)
      if (AUTH_PROBE_URLS.has(original.url ?? '')) {
        return Promise.reject(error)
      }

      if (isRefreshing) {
        // Attendre que le refresh en cours se termine
        const ok = await new Promise<boolean>(resolve => refreshQueue.push(resolve))
        if (!ok) return Promise.reject(error)
        return api.request(original)
      }

      isRefreshing = true
      try {
        await api.post('/auth/refresh')
        notifyQueue(true)
        original._retry = true
        return api.request(original)
      } catch {
        notifyQueue(false)
        // Refresh failed → log out
        window.dispatchEvent(new CustomEvent('laruche:session-expired'))
        return Promise.reject(error)
      } finally {
        isRefreshing = false
      }
    }

    return Promise.reject(error)
  }
)

// ── Helper d'extraction d'erreur ──────────────────────────────────
// Returns an i18n key (e.g. "errors.UNAUTHORIZED") to be translated by the caller.
export function extractApiError(error: unknown): string {
  if (axios.isAxiosError(error)) {
    const data = error.response?.data as ApiError | undefined
    if (data?.error) return `errors.${data.error}`
  }
  return 'errors.unknown'
}
