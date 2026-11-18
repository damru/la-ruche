import { expect, type Locator } from '@playwright/test'

/**
 * Helper transversal DDR-006 : valide la structure de feedback d'un
 * formulaire (CFormActions → CFormFeedback + bouton CBtn).
 *
 * Sans assert de position pixel ni de timing d'animation (fragiles) —
 * la position est garantie par CFormActions (CFormFeedback ne s'utilise
 * jamais directement, garde-fou statique ddr006.spec.ts).
 *
 * Le feedback inline s'auto-efface après 3 s (SUCCESS_AUTO_CLEAR_MS) :
 * les asserts succès doivent suivre la soumission immédiatement.
 */

const FEEDBACK = '.form-feedback'

export async function expectFeedback(
  scope: Locator,
  tone: 'valid' | 'err',
  message?: string | RegExp,
): Promise<void> {
  const feedback = scope.locator(`${FEEDBACK}.tone--${tone}`)
  await expect(feedback).toBeVisible()
  if (message !== undefined) {
    await expect(feedback).toContainText(message)
  }
}

export async function expectNoFeedback(scope: Locator): Promise<void> {
  await expect(scope.locator(FEEDBACK)).toHaveCount(0)
}

/** Bouton principal en attente : CBtn loading = spinner + pointer-events, pas d'attribut disabled. */
export async function expectSubmitPending(scope: Locator): Promise<void> {
  const submit = scope.locator('button[type="submit"]')
  await expect(submit.locator('.c-btn__spinner')).toBeVisible()
}

/** DDR-006 : après succès, le bouton principal passe en état `closing`. */
export async function expectSubmitClosing(scope: Locator): Promise<void> {
  const submit = scope.locator('button[type="submit"].c-btn--closing')
  await expect(submit).toBeVisible()
}

/** Fenêtre de succès auto-close DDR-006 : la modale se ferme seule. */
export async function expectAutoClose(scope: Locator, timeoutMs = 5_000): Promise<void> {
  await expect(scope).toBeHidden({ timeout: timeoutMs })
}
