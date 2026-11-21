import { test, expect } from '../../fixtures/auth'
import { apiPost, e2eName } from '../../data/api'
import type { Task } from '../../../src/types/domain'

/**
 * EP-007-S07 — take back a task from the quote phase (TasksView).
 *
 * The quote-phase statuses are only reachable through the vendor flow (no
 * manager-side endpoint transitions a task into them), so the list response
 * is intercepted and a real task is re-labelled `awaiting_quote`. The
 * take-back POST itself stays real: the backend validates the transition.
 */

const TAKE_BACK_LABEL = 'Reprendre en main'
const TAKE_BACK_SUCCESS = 'Tâche reprise en main'

async function mockTaskList(page: import('@playwright/test').Page, tasks: Task[]): Promise<void> {
  await page.route('**/api/v1/tasks?*', route =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ items: tasks, total: tasks.length }) }),
  )
}

test.describe('take back a quote-phase task', () => {
  let authPage: import('@playwright/test').Page

  test.beforeEach(async ({ auth }, testInfo) => {
    const { page } = await auth('manager')
    authPage = page
    testInfo.attach('role', { body: 'manager' })
  })

  test.afterEach(async () => {
    await authPage.context().close()
  })

  test('take-back button only on quote-phase cards', async () => {
    const quoteTask = await apiPost<Task>(authPage, '/tasks', { title: e2eName(test.info(), 'quote task'), description: 'e2e' })
    const openTask = await apiPost<Task>(authPage, '/tasks', { title: e2eName(test.info(), 'open task'), description: 'e2e' })
    await mockTaskList(authPage, [
      { ...quoteTask, status: 'awaiting_quote' },
      openTask,
    ])

    await authPage.goto('/tasks')
    await expect(authPage.locator('.task-card').first()).toBeVisible()
    await expect(authPage.locator(`[title="${TAKE_BACK_LABEL}"]`)).toHaveCount(1)
  })

  test('confirm dialog cancel -> no request', async () => {
    const task = await apiPost<Task>(authPage, '/tasks', { title: e2eName(test.info(), 'quote task'), description: 'e2e' })
    await mockTaskList(authPage, [{ ...task, status: 'awaiting_quote' }])

    await authPage.goto('/tasks')
    await expect(authPage.locator('.task-card').first()).toBeVisible()

    let postEmitted = false
    authPage.on('request', req => {
      if (req.url().includes('/take-back') && req.method() === 'POST') postEmitted = true
    })

    await authPage.locator(`[title="${TAKE_BACK_LABEL}"]`).first().click()
    const dialog = authPage.locator('.c-modal__content').last()
    await expect(dialog).toBeVisible()
    await dialog.locator('button', { hasText: 'Annuler' }).click()

    await authPage.waitForTimeout(500)
    expect(postEmitted).toBe(false)
    await expect(authPage.locator('.c-modal__content')).toHaveCount(0)
  })

  test('confirm -> POST take-back -> success alert', async () => {
    const task = await apiPost<Task>(authPage, '/tasks', { title: e2eName(test.info(), 'quote task'), description: 'e2e' })
    await mockTaskList(authPage, [{ ...task, status: 'awaiting_quote' }])
    await authPage.route('**/api/v1/tasks/*/take-back', route =>
      route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ ...task, status: 'in_progress' }) }),
    )

    await authPage.goto('/tasks')
    await expect(authPage.locator('.task-card').first()).toBeVisible()

    const [resp] = await Promise.all([
      authPage.waitForResponse(r => r.url().includes('/take-back') && r.request().method() === 'POST'),
      (async () => {
        await authPage.locator(`[title="${TAKE_BACK_LABEL}"]`).first().click()
        const dialog = authPage.locator('.c-modal__content').last()
        await dialog.locator('button', { hasText: TAKE_BACK_LABEL }).click()
      })(),
    ])
    expect(resp.status()).toBe(200)
    expect(resp.json()).resolves.toMatchObject({ status: 'in_progress' })

    await expect(authPage.getByText(TAKE_BACK_SUCCESS)).toBeVisible()
  })

  test('server failure -> error alert', async () => {
    const task = await apiPost<Task>(authPage, '/tasks', { title: e2eName(test.info(), 'quote task'), description: 'e2e' })
    await mockTaskList(authPage, [{ ...task, status: 'awaiting_quote' }])
    await authPage.route('**/api/v1/tasks/*/take-back', route =>
      route.fulfill({ status: 500, contentType: 'application/json', body: '{"error":"boom"}' }),
    )

    await authPage.goto('/tasks')
    await expect(authPage.locator('.task-card').first()).toBeVisible()

    await authPage.locator(`[title="${TAKE_BACK_LABEL}"]`).first().click()
    const dialog = authPage.locator('.c-modal__content').last()
    await dialog.locator('button', { hasText: TAKE_BACK_LABEL }).click()

    await expect(authPage.locator('.c-alert.tone--err')).toBeVisible()
  })
})
