import { test, expect } from '../../fixtures/auth'
import { expectAutoClose, expectFeedback, expectSubmitClosing } from '../../helpers-feedback'
import { JudicialProceedingsPage } from '../../pages/judicial-proceedings'
import { e2eName } from '../../data/api'

/**
 * Actions registre judiciaire — succès (feedback, closing, auto-close,
 * table rafraîchie) et échec serveur (feedback tone--err).
 */

test.describe('actions registre judiciaire', () => {
  let page_o: JudicialProceedingsPage

  test.beforeEach(async ({ auth }, testInfo) => {
    const { page } = await auth('manager')
    page_o = new JudicialProceedingsPage(page)
    testInfo.attach('role', { body: 'manager' })
  })

  test.afterEach(async () => {
    await page_o.page.context().close()
  })

  test('création : succès, bouton closing, auto-close, ligne dans la table', async ({ }, testInfo) => {
    const subject = e2eName(testInfo, 'procédure créée')
    await page_o.goto()
    await page_o.openCreateModal()
    await page_o.fill({
      subject,
      claimant: 'Copropriété Test',
      defendant: 'Prestataire Test',
      openedAt: '2026-01-15',
    })
    const [resp] = await Promise.all([
      page_o.page.waitForResponse(r => r.url().includes('/api/v1/judicial-proceedings') && r.request().method() === 'POST'),
      page_o.submit(),
    ])
    expect(resp.status()).toBe(201)

    await expectFeedback(page_o.modal, 'valid')
    await expectSubmitClosing(page_o.modal)
    await expectAutoClose(page_o.modal)
    await expect(page_o.rowBySubject(subject)).toBeVisible()
  })

  test('échec serveur : feedback tone--err, modale maintenue ouverte', async () => {
    await page_o.goto()
    await page_o.openCreateModal()
    await page_o.fill({
      subject: 'e2e-errsrv-uniq-42',
      claimant: 'A',
      defendant: 'B',
      openedAt: '2026-01-15',
    })

    await page_o.page.route(
      url => url.pathname.includes('/judicial-proceedings'),
      route => route.fulfill({ status: 422, contentType: 'application/json', body: '{"error":"payload invalide"}' }),
    )
    await page_o.submit()

    await expectFeedback(page_o.modal, 'err')
    await expect(page_o.modal).toBeVisible()
  })
})
