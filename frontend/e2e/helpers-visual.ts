import type { Page } from '@playwright/test'

/**
 * Frozen clock for visual captures — single source of truth. The CI workflow
 * (visual-preview.yml) anchors the demo seed's NOW()/CURRENT_DATE to this
 * same instant; otherwise every re-seed churns the screenshots. The workflow
 * reads this constant with sed: keep the export on exactly one line.
 */
export const FIXED_NOW_ISO = '2026-01-15T09:00:00Z'

/**
 * Settle a page to its deterministic terminal state before capture: fonts
 * loaded, <img> at loaded/error terminal state (S3 404s included), every
 * animation/transition frozen (reducedMotion only covers CSS that responds
 * to it — pulsing badges etc. would capture at a random phase), one painted
 * frame. Without this, fast CI runners capture mid-swap pages that churn
 * run to run.
 */
export async function settlePage(page: Page): Promise<void> {
  await page.evaluate(async () => {
    await document.fonts.ready
    await Promise.all(
      Array.from(document.images).map((img) =>
        img.complete
          ? Promise.resolve()
          : new Promise((res) => {
              img.onload = img.onerror = res
            }),
      ),
    )
  })
  await page.addStyleTag({
    content:
      '*, *::before, *::after { animation: none !important; transition: none !important; caret-color: transparent !important; }',
  })
  await page.evaluate(
    () =>
      new Promise((res) =>
        requestAnimationFrame(() => requestAnimationFrame(res)),
      ),
  )
}
