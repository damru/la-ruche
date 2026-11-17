import { test, expect } from '../fixtures/auth'
import fs from 'node:fs'
import path from 'node:path'
import { APP_ROUTES } from '../data/access-matrix'

/**
 * Cohérence matrice ↔ routeur (plan Phase 1 §1) : le routeur n'est pas
 * importable en Node (createWebHistory au niveau module, routes non exporté),
 * donc relecture texte de src/router/index.ts + extraction regex des paths
 * enfant du layout authentifié. Toute nouvelle route non déclarée dans la
 * matrice fait échouer ce test.
 */

const routerSource = fs.readFileSync(
  path.resolve(import.meta.dirname, '../../src/router/index.ts'),
  'utf-8',
)

/** Paths enfant du layout authentifié, extraits du source (ordre du fichier). */
function extractAuthChildPaths(): string[] {
  // Fenêtre : du layout `path: '/'` jusqu'au bloc erreurs (/403).
  const start = routerSource.indexOf("path: '/'")
  const end = routerSource.indexOf("path: '/403'")
  const slice = routerSource.slice(start, end)
  const out: string[] = []
  for (const m of slice.matchAll(/path:\s*'([^']+)'/g)) {
    const p = m[1]
    if (p === '' || p.startsWith('profile/')) continue // redirect '' et legacy
    if (p === 'settings') continue // redirect pur
    out.push('/' + p) // routes paramétrées conservées (match matrice §détail)
  }
  return out
}

/**
 * Une entrée de matrice correspond au routeur si le path est exact, ou si une
 * route paramétrée du routeur produit le même squelette statique
 * (ex. /residents/:id/charges ↔ /residents/<uuid>/charges).
 */
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

    // Dette tracée : routes de détail paramétrées, sans instance stable en
    // lecture seule — couvertes en Phase 2 (entités créées par API).
    // Une route paramétrée NOUVELLE (hors cette liste) doit échouer ici.
    const delegatedPhase2 = [
      '/meetings/:id',
      '/tasks/:id',
      '/contractors/:id',
      '/contracts/:id',
    ]

    const uncovered = routerPaths.filter((p) => {
      if (p === '//') return false // redirect '' du layout (→ /dashboard)
      if (p === '/residents/:id/charges') {
        // Couverte par l'entrée concrète de la matrice (unité seedée).
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
