import { test, expect } from '../fixtures/auth'
import { AxeBuilder } from '@axe-core/playwright'
import fs from 'node:fs'
import path from 'node:path'
import { APP_ROUTES } from '../data/access-matrix'

const EXCLUSIONS_PATH = path.resolve(import.meta.dirname, '../.axe-exclusions.json')

const seenViolationRules = new Set<string>()

type Exclusions = Record<string, string[]>

function loadExclusions(): Exclusions {
  if (!fs.existsSync(EXCLUSIONS_PATH)) return {}
  return JSON.parse(fs.readFileSync(EXCLUSIONS_PATH, 'utf-8')) as Exclusions
}

const VIEWPORTS = [
  { name: 'desktop', width: 1440, height: 900 },
  { name: 'tablet', width: 768, height: 1024 },
  { name: 'mobile', width: 375, height: 812 },
] as const

const THEMES = ['light', 'dark'] as const

function filterViolations(
  violations: Awaited<ReturnType<AxeBuilder['analyze']>>['violations'],
  exclusions: Exclusions,
) {
  const isDocKey = (id: string) => id.startsWith('$')
  const remaining = violations
    .map((v) => {
      const excluded = (exclusions[v.id] ?? []).filter((e) => !isDocKey(e))
      const nodes = (v.nodes ?? []).filter(

        (n) => !n.target.some((t) => excluded.some((e) => (t as string).includes(e))),
      )
      return { id: v.id, count: nodes.length }
    })
    .filter((v) => v.count > 0)
  return remaining
}

test.describe('Accessibilité — axe-core WCAG 2.2 AA', () => {

  test.describe.configure({ mode: 'serial' })

  for (const vp of VIEWPORTS) {
    for (const theme of THEMES) {
      for (const route of APP_ROUTES.filter((r) => r.roles.includes('manager'))) {
        test(`axe ${theme}/${vp.name} : ${route.path}`, async ({ auth }) => {
          const { page, close } = await auth('manager')
          await page.setViewportSize({ width: vp.width, height: vp.height })
          await page.goto(route.path)
          await page.waitForLoadState('networkidle')

          await page.waitForFunction(
            () => !document.querySelector('.c-page-loader, [class*="skeleton"], .c-btn__spinner'),
            undefined,
            { timeout: 15_000 },
          ).catch(() => {})
          await page.waitForTimeout(400)

          await page.evaluate((t) => {
            document.documentElement.setAttribute('data-theme', t)
          }, theme)

          const builder = new AxeBuilder({ page })
            .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'])
            .exclude('.c-legal-notice')
          const results = await builder.analyze()
          for (const v of results.violations ?? []) seenViolationRules.add(v.id)

          const remaining = filterViolations(results.violations ?? [], loadExclusions())
          const summary = remaining
            .map((v) => `${v.id} ×${v.count}`)
            .join(', ')
          const details = remaining
            .map((v) => {
              const full = (results.violations ?? []).find((r) => r.id === v.id)
              const excluded = (loadExclusions()[v.id] ?? []).filter((e) => !e.startsWith('$'))
              return (full?.nodes ?? [])
                .filter((n) => !n.target.some((t) => excluded.some((e) => (t as string).includes(e))))
                .slice(0, 5)
                .map((n) => `${v.id}: ${n.target.join(' ')} | ${(n.html ?? '').slice(0, 120)}`)
            })
            .flat()
          expect(
            remaining,
            `violations axe (${theme}/${vp.name}) sur ${route.path} : ${summary}\n${details.join('\n')}`,
          ).toEqual([])

          await close()
        })
      }
    }
  }

  test('exclusions axe toutes utilisées (pas de dette périmée)', async () => {
    const exclusions = loadExclusions()
    const ids = Object.keys(exclusions).filter((id) => !id.startsWith('$'))
    if (ids.length === 0) return

    for (const id of ids) {
      expect(
        seenViolationRules.has(id),
        `exclusion axe périmée : la règle "${id}" n'a produit aucune violation sur les pages scannées — retire-la de .axe-exclusions.json`,
      ).toBe(true)
    }
  })
})
