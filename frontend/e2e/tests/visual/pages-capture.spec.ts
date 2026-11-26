import { test, expect } from '@playwright/test'
import { APP_ROUTES, allowedAccounts } from '../../data/access-matrix'
import { FIXED_NOW_ISO, settlePage } from '../../helpers-visual'

// Phase 3 — before/after screen comparison in the PR review. NOT an
// assertion suite: screenshots one page per route x viewport x theme; the
// CI job commits changed PNGs to the PR branch so the diff shows natively
// in Files Changed (Swipe). CI x86 only — Pi ARM64 renders differently.
const FIXED_NOW = new Date(FIXED_NOW_ISO)

const VIEWPORTS = [
  { name: '1440', width: 1440, height: 900 },
  { name: '375', width: 375, height: 812 },
] as const

const THEMES = ['light', 'dark'] as const

function slugify(path: string): string {
  return path.replace(/[^a-z0-9]+/gi, '-').replace(/(^-|-$)/g, '')
}

for (const route of APP_ROUTES) {
  const role = allowedAccounts(route.roles)[0]
  const slug = slugify(route.path)

  for (const viewport of VIEWPORTS) {
    for (const theme of THEMES) {
      test(`capture ${slug} -- ${viewport.name} -- ${theme}`, async ({ browser }) => {
        const context = await browser.newContext({
          storageState: `e2e/.auth/${role}.json`,
          viewport: { width: viewport.width, height: viewport.height },
          colorScheme: theme,
          reducedMotion: 'reduce',
        })
        const page = await context.newPage()

        await page.clock.install({ time: FIXED_NOW })
        // Before app boot so the anti-flash script (index.html) picks it up.
        await page.addInitScript((t) => window.localStorage.setItem('laruche-theme', t), theme)

        await page.goto(route.path)
        await page.waitForLoadState('networkidle')
        await settlePage(page)

        // Sanity guard: a page that failed to render must fail the capture.
        await expect(page.locator('.view-header').first()).toBeVisible()
        await expect(page.locator('.c-page-error')).toHaveCount(0)

        await page.addStyleTag({ content: '.c-bottom-nav { position: static !important; order: 3; }' })

        await page.screenshot({
          path: `e2e/visual/${slug}--${viewport.name}--${theme}.png`,
          fullPage: true,
        })

        for (const tab of route.captureTabs ?? []) {
          await page.getByRole('button', { name: tab.label, exact: true }).click()
          await page.waitForLoadState('networkidle')
          await settlePage(page)
          await page.screenshot({
            path: `e2e/visual/${slug}--tab-${tab.slug}--${viewport.name}--${theme}.png`,
            fullPage: true,
          })
        }

        if (route.captureGrid) {
          const gridToggle = page.getByRole('button', { name: 'Vue grille' })
          await gridToggle.click()
          await page.waitForLoadState('networkidle')
          await settlePage(page)
          await page.screenshot({
            path: `e2e/visual/${slug}--grid--${viewport.name}--${theme}.png`,
            fullPage: true,
          })
        }
        await context.close()
      })
    }
  }
}
