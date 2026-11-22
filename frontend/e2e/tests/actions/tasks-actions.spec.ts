import { test, expect } from '../../fixtures/auth'
import { expectAutoClose, expectFeedback, expectSubmitClosing, expectSubmitPending } from '../../helpers-feedback'
import { TasksPage } from '../../pages/tasks'
import { apiPost, e2eName } from '../../data/api'
import type { Task } from '../../../src/types/domain'

/**
 * Task actions — client validation, server failure, success with effects
 * (feedback, closing button, DDR-006 auto-close, refreshed list), edit,
 * take-back from the quote phase (EP-007-S07).
 *
 * Entities created via API (data/api.ts), unique name per worker. The only
 * `page.route` calls simulate a server failure or a quote-phase list —
 * minimal scope, target endpoint only.
 */

const TAKE_BACK_LABEL = 'Reprendre en main'
const TAKE_BACK_SUCCESS = 'Tâche reprise en main'

async function mockTaskList(page: import('@playwright/test').Page, tasks: Task[]): Promise<void> {
  await page.route('**/api/v1/tasks?*', route =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ items: tasks, total: tasks.length }) }),
  )
}

test.describe('task actions', () => {
  let tasks: TasksPage

  test.beforeEach(async ({ auth }, testInfo) => {
    const { page } = await auth('manager')
    tasks = new TasksPage(page)
    testInfo.attach('role', { body: 'manager' })
  })

  test.afterEach(async () => {
    await tasks.page.context().close()
  })

  test('client validation: empty title -> disabled button, no request', async () => {
    await tasks.goto()
    await tasks.openCreateModal()

    let postEmitted = false
    tasks.page.on('request', req => {
      if (req.url().includes('/api/v1/tasks') && req.method() === 'POST') postEmitted = true
    })

    await expect(tasks.modal.locator('button[type="submit"]')).toBeDisabled()
    expect(postEmitted).toBe(false)
    await tasks.page.waitForTimeout(500)
    expect(postEmitted).toBe(false)
  })

  test('server failure: tone--err feedback, modal kept open', async () => {
    await tasks.goto()
    await tasks.openCreateModal()
    await tasks.fillTitle('e2e-w-erreur-serveur')

    await tasks.page.route('**/api/v1/tasks', route =>
      route.fulfill({ status: 500, contentType: 'application/json', body: '{"error":"boom"}' }),
    )
    const [resp] = await Promise.all([
      tasks.page.waitForResponse(r => r.url().includes('/api/v1/tasks') && r.request().method() === 'POST'),
      tasks.submit(),
    ])
    expect(resp.status()).toBe(500)

    await expectFeedback(tasks.modal, 'err')
    await expect(tasks.modal).toBeVisible()
  })

  test('success: valid feedback, closing button, auto-close, refreshed list', async () => {
    const title = e2eName(test.info(), 'created task')
    await tasks.goto()
    await tasks.openCreateModal()
    await tasks.fillTitle(title)
    await tasks.fillDescription('description e2e')
    const [resp] = await Promise.all([
      tasks.page.waitForResponse(r => r.url().includes('/api/v1/tasks') && r.request().method() === 'POST'),
      tasks.submit(),
    ])
    expect(resp.status()).toBe(201)

    await expectFeedback(tasks.modal, 'valid')
    await expectSubmitClosing(tasks.modal)
    await expectAutoClose(tasks.modal)
    await expect(tasks.cardByTitle(title)).toBeVisible()
  })

  test('button disabled while the request is in flight', async ({ page }) => {
    await tasks.goto()
    await tasks.openCreateModal()
    await tasks.fillTitle(e2eName(test.info(), 'slow task'))

    await page.route('**/api/v1/tasks', async route => {
      await new Promise(r => setTimeout(r, 1_500))
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({ id: '00000000-0000-0000-0000-0000000000e2', title: 'x', status: 'open' }),
      })
    })
    const respP = tasks.page.waitForResponse(r => r.url().includes('/api/v1/tasks') && r.request().method() === 'POST')
    await tasks.submit()
    await expectSubmitPending(tasks.modal)
    await respP
    await expectAutoClose(tasks.modal)
  })

  test('edit via API: list refreshed with the new title', async () => {
    const title = e2eName(test.info(), 'task to edit')
    const created = await apiPost<{ id: string }>(tasks.page, '/tasks', { title })

    const newTitle = `${title} v2`
    const res = await tasks.page.request.put(`/api/v1/tasks/${created.id}`, {
      data: { title: newTitle },
    })
    expect(res.status()).toBe(204)

    await tasks.goto()
    await expect(tasks.cardByTitleExact(newTitle)).toBeVisible()
    await expect(tasks.cardByTitleExact(title)).toHaveCount(0)
  })

  // EP-007-S07 — quote-phase statuses are only reachable through the vendor
  // flow (no manager-side endpoint transitions a task into them), so the
  // list response is intercepted and a real task is re-labelled
  // `awaiting_quote`. The take-back POST itself stays real: the backend
  // validates the transition.

  test('take-back button only on quote-phase cards', async () => {
    const quoteTask = await apiPost<Task>(tasks.page, '/tasks', { title: e2eName(test.info(), 'quote task'), description: 'e2e' })
    const openTask = await apiPost<Task>(tasks.page, '/tasks', { title: e2eName(test.info(), 'open task'), description: 'e2e' })
    await mockTaskList(tasks.page, [
      { ...quoteTask, status: 'awaiting_quote' },
      openTask,
    ])

    await tasks.goto()
    await expect(tasks.page.locator('.task-card').first()).toBeVisible()
    await expect(tasks.page.locator(`[title="${TAKE_BACK_LABEL}"]`)).toHaveCount(1)
  })

  test('take-back: confirm dialog cancel -> no request', async () => {
    const task = await apiPost<Task>(tasks.page, '/tasks', { title: e2eName(test.info(), 'quote task'), description: 'e2e' })
    await mockTaskList(tasks.page, [{ ...task, status: 'awaiting_quote' }])

    await tasks.goto()
    await expect(tasks.page.locator('.task-card').first()).toBeVisible()

    let postEmitted = false
    tasks.page.on('request', req => {
      if (req.url().includes('/take-back') && req.method() === 'POST') postEmitted = true
    })

    await tasks.page.locator(`[title="${TAKE_BACK_LABEL}"]`).first().click()
    const dialog = tasks.page.locator('.c-modal__content').last()
    await expect(dialog).toBeVisible()
    await dialog.locator('button', { hasText: 'Annuler' }).click()

    await tasks.page.waitForTimeout(500)
    expect(postEmitted).toBe(false)
    await expect(tasks.page.locator('.c-modal__content')).toHaveCount(0)
  })

  test('take-back: confirm -> POST -> success alert', async () => {
    const task = await apiPost<Task>(tasks.page, '/tasks', { title: e2eName(test.info(), 'quote task'), description: 'e2e' })
    await mockTaskList(tasks.page, [{ ...task, status: 'awaiting_quote' }])
    await tasks.page.route('**/api/v1/tasks/*/take-back', route =>
      route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ ...task, status: 'in_progress' }) }),
    )

    await tasks.goto()
    await expect(tasks.page.locator('.task-card').first()).toBeVisible()

    const [resp] = await Promise.all([
      tasks.page.waitForResponse(r => r.url().includes('/take-back') && r.request().method() === 'POST'),
      (async () => {
        await tasks.page.locator(`[title="${TAKE_BACK_LABEL}"]`).first().click()
        const dialog = tasks.page.locator('.c-modal__content').last()
        await dialog.locator('button', { hasText: TAKE_BACK_LABEL }).click()
      })(),
    ])
    expect(resp.status()).toBe(200)
    expect(resp.json()).resolves.toMatchObject({ status: 'in_progress' })

    await expect(tasks.page.getByText(TAKE_BACK_SUCCESS)).toBeVisible()
  })

  test('take-back: server failure -> error alert', async () => {
    const task = await apiPost<Task>(tasks.page, '/tasks', { title: e2eName(test.info(), 'quote task'), description: 'e2e' })
    await mockTaskList(tasks.page, [{ ...task, status: 'awaiting_quote' }])
    await tasks.page.route('**/api/v1/tasks/*/take-back', route =>
      route.fulfill({ status: 500, contentType: 'application/json', body: '{"error":"boom"}' }),
    )

    await tasks.goto()
    await expect(tasks.page.locator('.task-card').first()).toBeVisible()

    await tasks.page.locator(`[title="${TAKE_BACK_LABEL}"]`).first().click()
    const dialog = tasks.page.locator('.c-modal__content').last()
    await dialog.locator('button', { hasText: TAKE_BACK_LABEL }).click()

    await expect(tasks.page.locator('.c-alert.tone--err')).toBeVisible()
  })
})
