import { test, expect } from '../../fixtures/auth'
import { expectAutoClose, expectFeedback, expectSubmitClosing } from '../../helpers-feedback'
import { JudicialProceedingsPage } from '../../pages/judicial-proceedings'
import { e2eName } from '../../data/api'

/**
 * Judicial register actions — success (feedback, closing, auto-close,
 * refreshed table) and server failure (tone--err feedback).
 */

test.describe('judicial register actions', () => {
  let page_o: JudicialProceedingsPage

  test.beforeEach(async ({ auth }, testInfo) => {
    const { page } = await auth('manager')
    page_o = new JudicialProceedingsPage(page)
    testInfo.attach('role', { body: 'manager' })
  })

  test.afterEach(async () => {
    await page_o.page.context().close()
  })

  test('create: success, closing button, auto-close, table row', async () => {
    const subject = e2eName(test.info(), 'created proceeding')
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

  test('server failure: tone--err feedback, modal kept open', async () => {
    await page_o.goto()
    await page_o.openCreateModal()
    await page_o.fill({
      subject: 'e2e-server-failure-42',
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
