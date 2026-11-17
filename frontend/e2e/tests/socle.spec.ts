import { test, expect } from '../fixtures/auth'
import { ACCOUNTS, DEV_PASSWORD, type AuthRole } from '../fixtures/accounts'

/**
 * Test de validation du socle (Phase 0) : prouve que
 * webServer + backend + seed + auth par rôle tiennent ensemble.
 *
 * Parcours : login UI → dashboard rendu → logout (/profile) → /login.
 * Sélecteurs basés sur LoginView réel : #email, #password, submit CBtn,
 * erreur via CAlert variant="error" ("Email ou mot de passe incorrect").
 */

test.describe('Socle E2E — login par rôle', () => {
  for (const role of Object.keys(ACCOUNTS) as AuthRole[]) {
    test(`login UI → dashboard → logout (${role})`, async ({ page }) => {
      await page.goto('/login')

      // Formulaire de login (LoginView : ids #email / #password).
      await page.locator('#email').fill(ACCOUNTS[role])
      await page.locator('#password').fill(DEV_PASSWORD)
      await page.locator('button[type="submit"]').click()

      // Redirection vers le dashboard, page rendue.
      // Titre réel : `${t('nav.dashboard')} — La Ruche` (router/index.ts afterEach).
      await expect(page).toHaveURL(/\/dashboard/)
      await expect(page).toHaveTitle(/Tableau de bord — La Ruche/)
      await expect(page.locator('.c-page-error')).toHaveCount(0)

      // Logout depuis /profile → retour au login.
      // CBtn (variant ghost) rend le title "Se déconnecter" en texte du bouton.
      await page.goto('/profile')
      await page.getByRole('button', { name: /se déconnecter|log ?out/i }).click()
      await expect(page).toHaveURL(/\/login/, { timeout: 15_000 })
    })
  }

  test('session restaurée sans login UI : storageState manager → dashboard', async ({ auth }) => {
    // Valide le mécanisme auth par rôle : le contexte storageState donne
    // accès direct au dashboard, sans passer par le formulaire de login.
    const { page, close } = await auth('manager')
    await page.goto('/dashboard')
    await expect(page).toHaveURL(/\/dashboard/)
    await expect(page).toHaveTitle(/Tableau de bord — La Ruche/)
    await expect(page.locator('body')).not.toContainText('CPageError')
    await close()
  })

  test('échec de login : mauvais mot de passe → erreur, pas de redirection', async ({ page }) => {
    await page.goto('/login')
    await page.locator('#email').fill(ACCOUNTS.manager)
    await page.locator('#password').fill('MauvaisMotDePasse1!')
    await page.locator('button[type="submit"]').click()

    // Reste sur /login ; CAlert variant="error" affiche le message
    // (clé i18n errors.unauthorized : "Identifiants incorrects ou session expirée").
    await expect(page).toHaveURL(/\/login/)
    await expect(page.getByText('Identifiants incorrects ou session expirée')).toBeVisible()
  })
})
