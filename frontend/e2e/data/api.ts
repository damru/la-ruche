import type { APIResponse, Page, TestInfo } from '@playwright/test'

/**
 * Client API e2e : passe par le request context de la page (même origine,
 * le proxy Vite /api → backend :8080). Les cookies httpOnly du storageState
 * authentifient chaque appel — le rôle du test pilote ses permissions.
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
 * Nom unique pour une entité créée par un test : préfixe worker + run,
 * contre les collisions d'unicité quand fullyParallel partage la base.
 */
export function e2eName(testInfo: TestInfo, base: string): string {
  const seq = (e2eSeq = (e2eSeq + 1) % 1_000_000).toString(36).padStart(3, '0')
  const run = Date.now().toString(36)
  return `e2e-w${testInfo.workerIndex}-${run}${seq}-${base}`
}
