import { expect, type Locator } from '@playwright/test'

/**
 * Form UI effects (DDR-006): feedback, pending, closing, auto-close.
 * No position or animation asserts — position is guaranteed by CFormActions
 * (ddr006 guard). Feedback auto-clears after 3 s: asserts must follow the
 * submission immediately.
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

export async function expectSubmitPending(scope: Locator): Promise<void> {
  const submit = scope.locator('button[type="submit"]')
  await expect(submit.locator('.c-btn__spinner')).toBeVisible()
}

export async function expectSubmitClosing(scope: Locator): Promise<void> {
  const submit = scope.locator('button[type="submit"].c-btn--closing')
  await expect(submit).toBeVisible()
}

export async function expectAutoClose(scope: Locator, timeoutMs = 5_000): Promise<void> {
  await expect(scope).toBeHidden({ timeout: timeoutMs })
}
