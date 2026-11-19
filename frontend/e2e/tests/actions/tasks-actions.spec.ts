import { test, expect } from '../../fixtures/auth'
import { expectAutoClose, expectFeedback, expectSubmitClosing, expectSubmitPending } from '../../helpers-feedback'
import { TasksPage } from '../../pages/tasks'
import { apiPost, e2eName } from '../../data/api'

/**
 * Actions tâches — validation client, échec serveur, succès avec effets
 * (feedback, bouton closing, auto-close DDR-006, liste rafraîchie),
 * modification.
 *
 * Entités créées par API (data/api.ts), nom unique par worker. Le seul
 * `page.route` sert à simuler l'échec serveur — scope minimal, endpoint
 * concerné uniquement.
 */

test.describe('actions tâches', () => {
  let tasks: TasksPage

  test.beforeEach(async ({ auth }, testInfo) => {
    const { page } = await auth('manager')
    tasks = new TasksPage(page)
    testInfo.attach('role', { body: 'manager' })
  })

  test.afterEach(async () => {
    await tasks.page.context().close()
  })

  test('validation client : titre vide → bouton disabled, aucune requête', async () => {
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

  test('échec serveur : feedback tone--err, modale maintenue ouverte', async () => {
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

  test('succès : feedback valid, bouton closing, auto-close, liste rafraîchie', async (_, testInfo) => {
    const title = e2eName(testInfo, 'création tâche')
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

  test('bouton disabled pendant la requête en cours', async ({ page }, testInfo) => {
    await tasks.goto()
    await tasks.openCreateModal()
    await tasks.fillTitle(e2eName(testInfo, 'tâche lente'))

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

  test('modification par API : liste rafraîchie au nouveau titre', async (_, testInfo) => {
    const title = e2eName(testInfo, 'tâche à modifier')
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
})
