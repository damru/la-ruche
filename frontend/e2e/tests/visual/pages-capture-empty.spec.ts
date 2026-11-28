import { test, expect } from '@playwright/test'
import { APP_ROUTES } from '../../data/access-matrix'
import { FIXED_NOW_ISO, settlePage } from '../../helpers-visual'

/**
 * Empty-state captures: one dedicated pass after the seeded captures, on a
 * database whose entities were truncated (accounts and building survive, so
 * authenticated pages still render). Separate project because the reset
 * destroys the data the main capture pass needs.
 */
const FIXED_NOW = new Date(FIXED_NOW_ISO)

const VIEWPORTS = [
  { name: '1440', width: 1440, height: 900 },
  { name: '375', width: 375, height: 812 },
] as const

const THEMES = ['light', 'dark'] as const

const EMPTY_ROUTES = APP_ROUTES.filter(route => route.captureEmpty)

test.describe.configure({ mode: 'serial' })

test.beforeAll(async ({ browser }) => {
  const context = await browser.newContext({ storageState: 'e2e/.auth/manager.json' })
  const page = await context.newPage()
  const reset = await page.request.post('/api/v1/admin/reset-data')
  expect(reset.status(), 'reset-data doit vider les entités du seed').toBe(204)
  await context.close()
})

for (const route of EMPTY_ROUTES) {
  const slug = route.path.replace(/[^a-z0-9]+/gi, '-').replace(/(^-|-$)/g, '')

  for (const viewport of VIEWPORTS) {
    for (const theme of THEMES) {
      test(`capture ${slug} -- empty -- ${viewport.name} -- ${theme}`, async ({ browser }) => {
        const context = await browser.newContext({
          storageState: 'e2e/.auth/manager.json',
          viewport: { width: viewport.width, height: viewport.height },
          colorScheme: theme,
          reducedMotion: 'reduce',
        })
        const page = await context.newPage()

        await page.clock.install({ time: FIXED_NOW })
        await page.addInitScript((t) => window.localStorage.setItem('laruche-theme', t), theme)

        await page.goto(route.path)
        await page.waitForLoadState('networkidle')
        await settlePage(page)

        await expect(page.locator('.c-page-header').first()).toBeVisible()
        await expect(page.locator('.c-page-error')).toHaveCount(0)

        await page.addStyleTag({ content: '.c-bottom-nav { position: static !important; order: 3; }' })

        await page.screenshot({
          path: `e2e/visual/${slug}--empty--${viewport.name}--${theme}.png`,
          fullPage: true,
        })
        await context.close()
      })
    }
  }
}
