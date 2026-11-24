import { test, expect, type Page, type Route } from '@playwright/test'

/**
 * DDR-005 — the 6 network states, via `page.route()` interception.
 *
 * `mock` project: no database, no backend. Only the Vite app runs.
 * A catch-all `200 []` intercepts every `/api/v1/**` (stores call
 * users/contractors/counters on mount); each test then overrides the
 * endpoint it targets — Playwright gives priority to the last
 * registered handler.
 */

const FAKE_USER = {
  user_id: '00000000-0000-0000-0000-0000000000aa',
  building_id: '00000000-0000-0000-0000-0000000000bb',
  roles: ['manager'],
  last_name: 'E2E',
  first_name: 'Mock',
  email: 'mock@laruche.test',
  phone: null,
}

const TASK_LIST = { items: [], total: 0 }

function fulfillJson(route: Route, status: number, body: unknown): Promise<void> {
  return route.fulfill({ status, contentType: 'application/json', body: JSON.stringify(body) })
}

async function mockApi(page: Page): Promise<void> {
  await page.route('**/api/v1/**', route => fulfillJson(route, 200, []))
  await page.route('**/api/v1/runtime', route => fulfillJson(route, 200, { demo_mode: true }))
  await page.route('**/api/v1/building', route => fulfillJson(route, 200, {}))
  await page.route('**/api/v1/onboarding/status', route => fulfillJson(route, 200, { needs_onboarding: false }))
  await page.route('**/api/v1/auth/me', route => fulfillJson(route, 200, FAKE_USER))
  await page.route('**/api/v1/tasks**', route => fulfillJson(route, 200, TASK_LIST))
  await page.route('**/api/v1/tasks/status-counters', route => fulfillJson(route, 200, []))
}

function failTasks(page: Page, status: number | 'abort'): void {
  page.route('**/api/v1/tasks?*', async route => {
    if (status === 'abort') await route.abort('connectionreset')
    else await fulfillJson(route, status, { error: 'boom' })
  })
}

test.describe('DDR-005 network states — /tasks', () => {
  test('500 on mount -> CPageError replaces the content', async ({ page }) => {
    await mockApi(page)
    failTasks(page, 500)
    await page.goto('/tasks')
    await expect(page.locator('.c-page-error')).toBeVisible()
    await expect(page.locator('.task-card')).toHaveCount(0)
  })

  test('retry after 500 -> content rendered', async ({ page }) => {
    await mockApi(page)
    let calls = 0
    page.route('**/api/v1/tasks?*', async route => {
      calls += 1
      if (calls <= 1) await fulfillJson(route, 500, { error: 'boom' })
      else await fulfillJson(route, 200, TASK_LIST)
    })
    await page.goto('/tasks')
    const retry = page.locator('.c-page-error button', { hasText: 'Réessayer' })
    await expect(retry).toBeVisible()
    await retry.click()
    await expect(page.locator('.c-page-error')).toBeHidden()
    await expect(page.locator('.c-empty-state').first()).toBeVisible()
  })

  test('slow response -> loading state visible', async ({ page }) => {
    await mockApi(page)
    page.route('**/api/v1/tasks?*', async route => {
      await new Promise(r => setTimeout(r, 2_000))
      await fulfillJson(route, 200, TASK_LIST)
    })
    await page.goto('/tasks')
    await expect(page.locator('.c-page-loader, .c-skeleton, .spinner').first()).toBeVisible()
    await expect(page.locator('.c-empty-state').first()).toBeVisible({ timeout: 10_000 })
  })

  test('empty list (200 []) -> dedicated empty state', async ({ page }) => {
    await mockApi(page)
    await page.goto('/tasks')
    await expect(page.locator('.c-empty-state').first()).toBeVisible()
  })

  test('network timeout (abort) -> same contract as 500', async ({ page }) => {
    await mockApi(page)
    failTasks(page, 'abort')
    await page.goto('/tasks')
    await expect(page.locator('.c-page-error')).toBeVisible()
  })

  test('non-critical call failing -> page still functional', async ({ page }) => {
    await mockApi(page)
    await page.route('**/api/v1/tasks/status-counters', route => fulfillJson(route, 500, { error: 'boom' }))
    await page.goto('/tasks')
    await expect(page.locator('.c-page-error')).toHaveCount(0)
    await expect(page.locator('.c-empty-state').first()).toBeVisible()
  })
})
