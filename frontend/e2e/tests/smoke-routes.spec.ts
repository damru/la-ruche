import { test, expect } from '../fixtures/auth'
import { frText } from '../helpers-i18n'
import { attachCollectors } from '../helpers-net'
import { APP_ROUTES, allowedAccounts, type AuthRole, type MatrixEntry } from '../data/access-matrix'

/**
 * Smoke routes × rôles (Phase 1, lecture seule).
 *
 * Pour chaque route autorisée d'un rôle : la page rend son contenu signature
 * (texte i18n du meta.title), zéro console.error inattendu, zéro réponse API
 * en erreur inattendue, et pas de CPageError (DDR-005).
 * Pour chaque route interdite : redirection /403.
 */

test.describe('Smoke — routes autorisées par rôle', () => {
  for (const route of APP_ROUTES) {
    for (const role of allowedAccounts(route.roles)) {
      test(`${route.path} rend pour ${role}`, async ({ auth }) => {
        const { page, close } = await auth(role)
        const net = attachCollectors(page)

        await page.goto(route.path)

        // Réseau au repos (vue-query) avant d'assert sur le contenu.
        await page.waitForLoadState('networkidle')

        // Contenu signature : texte i18n du meta.title visible dans la page
        // (pattern transverse .view-page/.view-header, cf. plan §2).
        const expected = frText(route.titleKey)
        await expect(page.locator('.view-header').first()).toBeVisible()
        await expect(page.getByText(expected).first()).toBeVisible()

        // DDR-005 : pas d'état d'erreur page sur le happy path.
        await expect(page.locator('.c-page-error')).toHaveCount(0)

        // Pas d'erreur console ni d'appel API en échec inattendu.
        expect(net.consoleErrors, `console.error sur ${route.path}`).toEqual([])
        const tolerated = (route.toleratedApi403 ?? {})
        const toleratedForRole = Object.entries(tolerated)
          .filter(([, roles]) => roles.includes(role))
          .map(([endpoint]) => endpoint)
        const unexpected = net.apiFailures.filter(
          (f) => !toleratedForRole.some((endpoint) => f.includes(endpoint)),
        )
        expect(unexpected, `réponses API en erreur sur ${route.path}`).toEqual([])

        await close()
      })
    }
  }
})

test.describe('Smoke — routes interdites par rôle', () => {
  const ALL: AuthRole[] = ['manager', 'boardMember', 'coOwner', 'tenant']
  for (const route of APP_ROUTES) {
    // hasAnyRole : les comptes non listés par allowedAccounts sont interdits.
    const denied = ALL.filter((r) => !allowedAccounts(route.roles).includes(r))
    for (const role of denied) {
      test(`${route.path} → /403 pour ${role}`, async ({ auth }) => {
        const { page, close } = await auth(role)
        const net = attachCollectors(page)

        await page.goto(route.path)
        await page.waitForLoadState('networkidle')

        if ((route.deniedBehavior ?? 'route403') === 'pageError') {
          // Route ouverte au guard (meta.roles permissifs) : le backend
          // renvoie 403 et la vue rend CPageError (DDR-005) — pas de
          // redirection /403.
          await expect(page.locator('.c-page-error')).toBeVisible()
          expect(net.apiFailures, `API en erreur sur ${route.path}`).toEqual([
            expect.stringMatching(/^403 /),
          ])
          await close()
          return
        }

        // Le guard renvoie { name: 'forbidden' } (pas de 404 — fuite d'info).
        // ForbiddenView rend errors.forbidden ("Accès interdit").
        await expect(page).toHaveURL(/\/403/)
        await expect(page.getByText(frText('errors.forbidden')).first()).toBeVisible()

        expect(net.apiFailures, `API en erreur sur ${route.path} (interdit)`).toEqual([])
        await close()
      })
    }
  }
})

// Ré-export pour lisibilité des imports de test.
export type { MatrixEntry }
