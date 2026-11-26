import { test, expect } from '../fixtures/auth'
import fs from 'node:fs'
import path from 'node:path'
import { APP_ROUTES } from '../data/access-matrix'

const routerSource = fs.readFileSync(
  path.resolve(import.meta.dirname, '../../src/router/index.ts'),
  'utf-8',
)

function extractAuthChildPaths(): string[] {
  const start = routerSource.indexOf("path: '/'")
  const end = routerSource.indexOf("path: '/403'")
  const slice = routerSource.slice(start, end)
  const out: string[] = []
  for (const m of slice.matchAll(/path:\s*'([^']+)'/g)) {
    const p = m[1]
    if (p === '' || p.startsWith('profile/')) continue
    if (p === 'settings') continue
    out.push('/' + p)
  }
  return out
}

function matchesRouter(entryPath: string, routerPaths: string[]): boolean {
  if (routerPaths.includes(entryPath)) return true
  return routerPaths.some((rp) => {
    if (!rp.includes(':')) return false
    const pattern = rp.replace(/:[^/]+/g, '[^/]+')
    return new RegExp(`^${pattern}$`).test(entryPath)
  })
}

test.describe('Cohérence matrice d’accès ↔ routeur', () => {
  test('toute route du routeur est déclarée dans access-matrix', () => {
    const routerPaths = extractAuthChildPaths()
    const matrixPaths = APP_ROUTES.map((r) => r.path)

    const delegatedPhase2 = [
      '/meetings/:id',
      '/tasks/:id',
      '/contractors/:id',
      '/contracts/:id',
    ]

    const uncovered = routerPaths.filter((p) => {
      if (p === '//') return false
      if (p === '/residents/:id/charges') {
        return !matrixPaths.some((mp) => matchesRouter(mp, [p]))
      }
      if (p.includes(':')) {
        if (delegatedPhase2.includes(p)) return false
        return !matrixPaths.some((mp) => matchesRouter(mp, [p]))
      }
      return !matrixPaths.includes(p)
    })
    expect(
      uncovered,
      `routes du routeur absentes de access-matrix.ts — ajoute-les au smoke`,
    ).toEqual([])
  })

  test('toute entrée de la matrice existe dans le routeur', () => {
    const routerPaths = extractAuthChildPaths()
    const stale = APP_ROUTES.filter((r) => !matchesRouter(r.path, routerPaths))
    expect(
      stale.map((r) => r.path),
      `entrées de access-matrix.ts introuvables dans le routeur — nettoie`,
    ).toEqual([])
  })
})

test.describe('Cohérence des couleurs de titres', () => {
  test('un titre cliquable garde la couleur de titre, pas celle des liens', async ({ auth }) => {
    const { page, close } = await auth('coOwner')
    await page.goto('/reports')
    await expect(page.locator('main h3 a').first()).toBeVisible()

    const colors = await page.evaluate(() => {
      const link = document.querySelector('main h3 a') as HTMLElement
      const headingProbe = document.createElement('span')
      headingProbe.className = 'text-on-surface'
      const linkProbe = document.createElement('span')
      linkProbe.className = 'text-primary'
      document.body.append(headingProbe, linkProbe)
      const out = {
        link: getComputedStyle(link).color,
        heading: getComputedStyle(headingProbe).color,
        primary: getComputedStyle(linkProbe).color,
      }
      headingProbe.remove()
      linkProbe.remove()
      return out
    })

    expect(colors.link, 'le titre doit reprendre la couleur de titre').toBe(colors.heading)
    expect(colors.link, 'un titre ne doit pas prendre la couleur de lien').not.toBe(colors.primary)
    await close()
  })
})
