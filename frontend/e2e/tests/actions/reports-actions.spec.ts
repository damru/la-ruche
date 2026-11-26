import { test, expect } from '../../fixtures/auth'
import { apiPost, e2eName } from '../../data/api'

/**
 * Report detail (EP-017-S05): the lifecycle timeline comes from the API, the
 * conversion card links to the task only for roles that can open tasks/:id.
 * The chain is built through the API (report → convert) so assertions never
 * depend on seeded content.
 */

test.describe('report detail timeline', () => {
  test('resident sees the timeline and the conversion card without the task link', async ({ auth }, testInfo) => {
    const manager = await auth('manager')
    const resident = await auth('coOwner')
    const title = e2eName(testInfo, 'timeline')
    const report = await apiPost<{ id: string }>(manager.page, '/reports', { title })
    await apiPost(manager.page, `/reports/${report.id}/task`)

    await resident.page.goto(`/reports/${report.id}`)
    await expect(resident.page.getByRole('heading', { name: title })).toBeVisible()
    await expect(resident.page.getByText('Signalement créé')).toBeVisible()
    await expect(resident.page.getByText('Converti en tâche').first()).toBeVisible()
    await expect(resident.page.getByRole('link', { name: 'Voir la tâche' })).toHaveCount(0)
    await expect(resident.page.getByRole('link', { name: 'Ouvrir la tâche' })).toHaveCount(0)

    await manager.close()
    await resident.close()
  })

  test('manager follows the conversion link to the task timeline', async ({ auth }, testInfo) => {
    const manager = await auth('manager')
    const title = e2eName(testInfo, 'timeline')
    const report = await apiPost<{ id: string }>(manager.page, '/reports', { title })
    const converted = await apiPost<{ task_id: string }>(manager.page, `/reports/${report.id}/task`)

    await manager.page.goto(`/reports/${report.id}`)
    await manager.page.getByRole('link', { name: 'Voir la tâche' }).click()
    await expect(manager.page).toHaveURL(new RegExp(`/tasks/${converted.task_id}$`))
    await expect(manager.page.getByRole('heading', { name: title })).toBeVisible()
    await expect(manager.page.getByText('Tâche créée')).toBeVisible()
    await expect(manager.page.getByRole('link', { name: 'Ouvrir la tâche' })).toHaveCount(1)

    await manager.close()
  })
})
