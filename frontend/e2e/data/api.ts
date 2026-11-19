import type { APIResponse, Page, TestInfo } from '@playwright/test'

/**
 * E2E API client: goes through the page request context (same origin, the
 * Vite /api proxy -> backend :8080). The storageState httpOnly cookies
 * authenticate every call — the test role drives its permissions.
 */

let e2eSeq = 0

async function unwrap<T>(res: APIResponse): Promise<T> {
  if (!res.ok()) {
    const body = await res.text().catch(() => '')
    throw new Error(`API ${res.status()} ${res.url()}: ${body.slice(0, 300)}`)
  }
  if (res.status() === 204) return undefined as T
  return (await res.json()) as T
}

export function apiGet<T>(page: Page, path: string): Promise<T> {
  return page.request.get(`/api/v1${path}`).then(r => unwrap<T>(r))
}

export function apiPost<T>(page: Page, path: string, data?: unknown): Promise<T> {
  return page.request
    .post(`/api/v1${path}`, { data: data ?? {} })
    .then(r => unwrap<T>(r))
}

export function apiPut<T>(page: Page, path: string, data?: unknown): Promise<T> {
  return page.request
    .put(`/api/v1${path}`, { data: data ?? {} })
    .then(r => unwrap<T>(r))
}

export function apiDelete(page: Page, path: string): Promise<void> {
  return page.request.delete(`/api/v1${path}`).then(r => unwrap<void>(r))
}

export async function resetDemoData(page: Page): Promise<void> {
  await apiPost<void>(page, '/admin/reset-demo')
}

/**
 * Unique name for an entity created by a test: worker + run prefix, against
 * uniqueness collisions when fullyParallel shares the database.
 */
export function e2eName(testInfo: TestInfo, base: string): string {
  const seq = (e2eSeq = (e2eSeq + 1) % 1_000_000).toString(36).padStart(3, '0')
  const run = Date.now().toString(36)
  return `e2e-w${testInfo.workerIndex}-${run}${seq}-${base}`
}
