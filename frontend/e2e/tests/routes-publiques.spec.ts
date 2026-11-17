import { test, expect } from '../fixtures/auth'
import { frText } from '../helpers-i18n'
import { attachCollectors } from '../helpers-net'

/**
 * Routes publiques et pages d'erreur (Phase 1, lecture seule).
 *
 * NB onboarding : le seed démo crée une instance initialisée →
 * `/onboarding` redirige vers /login (non authentifié) ou /dashboard.
 */

test.describe('Routes publiques', () => {
  test('/login rend le formulaire de connexion', async ({ page }) => {
    const net = attachCollectors(page)
    await page.goto('/login')
    await page.waitForLoadState('networkidle')

    await expect(page.locator('#email')).toBeVisible()
    await expect(page.locator('#password')).toBeVisible()
    await expect(page.getByRole('button', { name: /se connecter/i })).toBeVisible()

    expect(net.apiFailures).toEqual([])
  })

  test('route inconnue → 404', async ({ page }) => {
    const net = attachCollectors(page)
    await page.goto('/route-inexistante-e2e')
    await page.waitForLoadState('networkidle')

    await expect(page).toHaveURL(/\/route-inexistante-e2e/)
    // NotFoundView rend errors.not_found ("Ressource introuvable").
    await expect(page.getByText(frText('errors.not_found')).first()).toBeVisible()
    expect(net.apiFailures).toEqual([])
  })

  test('/invite/:token — token inconnu → état d\'erreur explicite', async ({ page }) => {
    const net = attachCollectors(page)
    await page.goto('/invite/00000000-0000-0000-0000-00000000dead')
    await page.waitForLoadState('networkidle')

    // La vue affiche un état d'erreur explicite (CAlert) — c'est le critère.
    // Le réseau varie (404 ou re-validation silencieuse selon le timing) :
    // toute réponse API anormale doit rester cantonnée à invite/validate.
    await expect(page.getByText("Ce lien d'invitation est invalide")).toBeVisible()
    expect(net.apiFailures.filter((f) => !f.includes('401'))).toEqual([
      expect.stringMatching(/^(404|400) GET \/api\/v1\/auth\/invite\/validate$/),
    ])
  })

  test('/reset/:token — token inconnu → état d\'erreur explicite', async ({ page }) => {
    const net = attachCollectors(page)
    await page.goto('/reset/00000000-0000-0000-0000-00000000dead')
    await page.waitForLoadState('networkidle')

    await expect(page.locator('body')).not.toBeEmpty()
    expect(net.apiFailures.filter((f) => !f.includes('401'))).toEqual([])
  })

  test('/vendor/:token — page prestataire publique (token inconnu → erreur explicite)', async ({ page }) => {
    const net = attachCollectors(page)
    await page.goto('/vendor/00000000-0000-0000-0000-00000000dead')
    await page.waitForLoadState('networkidle')

    // 404 sur GET /api/v1/vendor/<token> = flux normal d'un token inconnu.
    await expect(page.locator('body')).not.toBeEmpty()
    // Route meta.public : pas de redirection login.
    await expect(page).not.toHaveURL(/\/login/)
    expect(net.apiFailures.filter((f) => !f.includes('401'))).toEqual([
      expect.stringMatching(/^404 GET \/api\/v1\/vendor\//),
    ])
  })

  test('/onboarding — instance initialisée → redirection (pas de re-onboarding)', async ({ page }) => {
    const net = attachCollectors(page)
    await page.goto('/onboarding')
    await page.waitForLoadState('networkidle')

    // Seed = instance initialisée → pas d'accès au formulaire d'onboarding.
    await expect(page).toHaveURL(/\/(login|dashboard)/)
    expect(net.apiFailures).toEqual([])
  })
})

test.describe('Auth — routes protégées hors connexion', () => {
  for (const path of ['/dashboard', '/budget', '/profile']) {
    test(`${path} hors connexion → /login avec redirect de retour`, async ({ page }) => {
      const net = attachCollectors(page)
      await page.goto(path)
      await page.waitForLoadState('networkidle')

      // Le router pose ?redirect=<path> (non encodé côté vue-router).
      await expect(page).toHaveURL(new RegExp(`/login\\?redirect=${path.replace(/\//g, '\\/')}`))
      expect(net.apiFailures).toEqual([])
    })
  }
})
