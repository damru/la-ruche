import { test, expect } from '@playwright/test'
import { APP_ROUTES, allowedAccounts } from '../../data/access-matrix'

/**
 * Phase 3 — Visual preview captures (repo-committed baselines).
 *
 * NOT an assertion suite: each test screenshots one stable layout page and
 * writes it to `e2e/visual/` with a deterministic filename. The CI job
 * (`visual-preview`) runs this project on PRs touching frontend/** and
 * commits changed PNGs to the branch, so image diffs appear natively in the
 * PR Files Changed tab (GitHub Swipe view). Screenshots are NEVER generated
 * locally (Pi ARM64 renders fonts/GPU differently) — CI x86 only.
 *
 * Determinism:
 * - Fixed clock (`page.clock.install`) — relative times ("in 2 months") and
 *   current-month headers are stable across runs. The CI workflow anchors
 *   the demo seed's NOW()/CURRENT_DATE to THE SAME instant (see
 *   visual-preview.yml "Pre-seed demo database") — the seed is time-relative,
 *   so anchoring only the browser clock would still churn every re-seed.
 * - `reducedMotion: 'reduce'` + theme forced via localStorage before app
 *   boot (`laruche-theme`, cf. index.html anti-flash script).
 * - Representative account only: first allowed account per route (manager
 *   for most routes) — role × page coverage is the smoke suite's job.
 */

const FIXED_NOW = new Date('2026-01-15T09:00:00Z')

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
        // Set before app boot so the anti-flash script (index.html) picks it up.
        await page.addInitScript((t) => window.localStorage.setItem('laruche-theme', t), theme)

        await page.goto(route.path)
        await page.waitForLoadState('networkidle')

        // networkidle is not enough on fast CI runners: fonts may still be
        // swapping in and <img> elements (S3-backed, 404 on CI) reach their
        // final loaded/error state at unpredictable times — capturing mid
        // swap makes the PNG unstable run to run. Settle everything to its
        // terminal state first.
        await page.evaluate(async () => {
          await document.fonts.ready
          await Promise.all(
            Array.from(document.images).map((img) =>
              img.complete
                ? Promise.resolve()
                : new Promise<void>((res) => {
                    img.onload = () => res()
                    img.onerror = () => res()
                  }),
            ),
          )
          await new Promise<void>((res) =>
            requestAnimationFrame(() => requestAnimationFrame(() => res())),
          )
        })

        // Sanity guard: a page that failed to render must fail the capture.
        await expect(page.locator('.view-header').first()).toBeVisible()
        await expect(page.locator('.c-page-error')).toHaveCount(0)

        await page.screenshot({
          path: `e2e/visual/${slug}--${viewport.name}--${theme}.png`,
        })

        await context.close()
      })
    }
  }
}
