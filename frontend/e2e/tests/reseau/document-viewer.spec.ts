import { test, expect, type Route } from '@playwright/test'

/**
 * EP-005-S11 — inline PDF viewer (CDocumentViewer) on AssemblyDetailView.
 *
 * `mock` project: no database, no backend, no S3. The catch-all `200 []`
 * intercepts every `/api/v1/**`; each test overrides the endpoints it
 * targets. The presigned-URL endpoints are mocked, and the storage host
 * serves a minimal PDF so the success path (iframe render) is covered
 * without a real bucket.
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

const MEETING_ID = '00000000-0000-0000-0000-00000000m001'
const PDF_DOC = {
  id: '00000000-0000-0000-0000-00000000d001',
  kind: 'attachment',
  mime_type: 'application/pdf',
  original_name: 'pv-ag.pdf',
}
const IMAGE_DOC = {
  id: '00000000-0000-0000-0000-00000000d002',
  kind: 'attachment',
  mime_type: 'image/png',
  original_name: 'photo.png',
}
const STORAGE_HOST = 'https://pdf-mock.test'
const PDF_BYTES = '%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\ntrailer\n<< /Root 1 0 R >>\n%%EOF'

function fulfillJson(route: Route, status: number, body: unknown): Promise<void> {
  return route.fulfill({ status, contentType: 'application/json', body: JSON.stringify(body) })
}

async function mockAssemblyPage(page: import('@playwright/test').Page, downloadUrlStatus = 200): Promise<void> {
  await page.route('**/api/v1/**', route => fulfillJson(route, 200, []))
  await page.route('**/api/v1/runtime', route => fulfillJson(route, 200, { demo_mode: true }))
  await page.route('**/api/v1/building', route => fulfillJson(route, 200, { name: 'Résidence E2E', shares_total: 100 }))
  await page.route('**/api/v1/onboarding/status', route => fulfillJson(route, 200, { needs_onboarding: false }))
  await page.route('**/api/v1/auth/me', route => fulfillJson(route, 200, FAKE_USER))
  await page.route('**/api/v1/meetings/' + MEETING_ID, route =>
    fulfillJson(route, 200, {
      id: MEETING_ID,
      meeting_type: 'annual',
      meeting_date: '2026-01-15T18:00:00Z',
      notice_date: '2026-01-02',
      location: 'Salle commune',
      status: 'closed',
      president_name: 'Président E2E',
      secretary_name: 'Secrétaire E2E',
      scrutineers: [],
      minutes_document_id: PDF_DOC.id,
      minutes_president_signed_at: null,
      minutes_secretary_signed_at: null,
      minutes_notified_at: null,
      value: null,
    }),
  )
  await page.route('**/api/v1/meetings/' + MEETING_ID + '/documents', route =>
    fulfillJson(route, 200, [PDF_DOC, IMAGE_DOC]),
  )
  await page.route('**/api/v1/meetings/' + MEETING_ID + '/attendance', route =>
    fulfillJson(route, 200, { entries: [], total_shares: 100, shares_present: 0, present_units: 0 }),
  )
  await page.route('**/api/v1/documents/*/download-url**', route => {
    if (downloadUrlStatus !== 200) return fulfillJson(route, downloadUrlStatus, { error: 'forbidden' })
    return fulfillJson(route, 200, { url: STORAGE_HOST + '/' + PDF_DOC.original_name })
  })
  // context-level: window.open creates a popup Page in the same context.
  await page.context().route(STORAGE_HOST + '/**', route =>
    route.fulfill({ status: 200, contentType: 'application/pdf', body: PDF_BYTES }),
  )
}

test.describe('EP-005-S11 inline PDF viewer', () => {
  test('pdf document -> viewer opens with inline iframe and enabled download', async ({ page }) => {
    await mockAssemblyPage(page)
    await page.goto('/meetings/' + MEETING_ID)
    await page.getByText(PDF_DOC.original_name).first().click()

    const viewer = page.locator('.c-modal__content').last()
    await expect(viewer.locator('iframe')).toBeVisible()
    await expect(viewer.locator('iframe')).toHaveAttribute('title', PDF_DOC.original_name)
    await expect(viewer.getByRole('button', { name: 'Télécharger' })).toBeEnabled()
  })

  test('presigned url failure -> fallback alert, download disabled', async ({ page }) => {
    await mockAssemblyPage(page, 403)
    await page.goto('/meetings/' + MEETING_ID)
    await page.getByText(PDF_DOC.original_name).first().click()

    const viewer = page.locator('.c-modal__content').last()
    await expect(viewer.getByText('Aperçu indisponible')).toBeVisible()
    await expect(viewer.getByRole('button', { name: 'Télécharger' })).toBeDisabled()
  })

  test('non-pdf document -> direct download, no viewer', async ({ page }) => {
    await mockAssemblyPage(page)
    await page.goto('/meetings/' + MEETING_ID)

    // 'noopener' window.open is not scriptable — assert the download-url
    // request (no disposition=inline means direct download, not the viewer).
    const [urlRequest] = await Promise.all([
      page.waitForRequest(r => r.url().includes(`/api/v1/documents/${IMAGE_DOC.id}/download-url`)),
      page.getByText(IMAGE_DOC.original_name).first().click(),
    ])
    expect(urlRequest.url()).not.toContain('disposition=inline')

    await expect(page.locator('.c-modal__content').last().locator('iframe')).toHaveCount(0)
  })
})
