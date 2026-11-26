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
  test('aucun titre cliquable ne prend la couleur des liens, sur toutes les pages', async ({ auth }) => {
    test.setTimeout(120_000)
    const { page, close } = await auth('manager')
    await page.goto('/dashboard')
    await expect(page.locator('main')).toBeVisible()

    const linkColour = await page.evaluate(() => {
      const probe = document.createElement('span')
      probe.className = 'text-primary'
      document.body.append(probe)
      const colour = getComputedStyle(probe).color
      probe.remove()
      return colour
    })

    const offenders: string[] = []
    for (const route of APP_ROUTES.filter(r => r.roles.includes('manager'))) {
      await page.goto(route.path)
      await page.waitForSelector('main', { timeout: 15_000 })
      const hits = await page.evaluate((primary: string) => {
        const out: string[] = []
        document.querySelectorAll('main h1 a, main h2 a, main h3 a, main h4 a').forEach(anchor => {
          if (getComputedStyle(anchor).color === primary) {
            out.push((anchor.textContent ?? '').trim().replace(/\s+/g, ' ').slice(0, 40))
          }
        })
        return out
      }, linkColour)
      if (hits.length) offenders.push(`${route.path} → ${hits.join(' | ')}`)
    }

    expect(offenders, 'titres cliquables rendus avec la couleur de lien').toEqual([])
    await close()
  })
})

test.describe('Cohérence du voile de modale', () => {
  test('l’overlay des modales consomme le token de scrim', async ({ auth }) => {
    const { page, close } = await auth('manager')
    await page.goto('/reports')
    await page.getByRole('button', { name: 'Nouveau signalement' }).click()
    await page.waitForSelector('.c-modal__overlay')

    const colours = await page.evaluate(() => {
      const overlay = document.querySelector('.c-modal__overlay') as HTMLElement
      const probe = document.createElement('div')
      probe.style.background = 'var(--color-scrim)'
      document.body.append(probe)
      const out = {
        overlay: getComputedStyle(overlay).backgroundColor,
        scrim: getComputedStyle(probe).backgroundColor,
      }
      probe.remove()
      return out
    })

    expect(colours.overlay, 'le voile doit être le token scrim, pas une couleur en dur').toBe(colours.scrim)
    await close()
  })
})
