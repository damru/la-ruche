import axios, { type AxiosInstance, type AxiosError } from 'axios'
import type { ApiError } from '@/types/domain'

// ── Instance principale ────────────────────────────────────────────
export const api: AxiosInstance = axios.create({
  baseURL: '/api/v1',
  withCredentials: true,  // cookies httpOnly JWT
  headers: { 'Content-Type': 'application/json' },
  timeout: 30_000,
})

// ── Refresh state (prevents duplicate requests) ──────────────────
let isRefreshing = false
let refreshQueue: Array<(ok: boolean) => void> = []

function notifyQueue(ok: boolean) {
  refreshQueue.forEach(resolve => resolve(ok))
  refreshQueue = []
}

// ── Response interceptor — transparent token refresh ─────────────────
api.interceptors.response.use(
  response => response,
  async (error: AxiosError<ApiError>) => {
    const original = error.config
    if (!original) return Promise.reject(error)

    const status = error.response?.status
    const errorCode = error.response?.data?.error

    // 401 TOKEN_INVALID → tenter un refresh
    if (status === 401 && errorCode === 'TOKEN_INVALID') {
      if (isRefreshing) {
        // Attendre que le refresh en cours se termine
        await new Promise<boolean>(resolve => refreshQueue.push(resolve))
        return api.request(original)
      }

      isRefreshing = true
      try {
        await api.post('/auth/refresh')
        notifyQueue(true)
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
