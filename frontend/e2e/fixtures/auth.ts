import { test as base, expect } from '@playwright/test'
import type { AuthRole } from './accounts'

/**
 * Fixture `auth` : contexte authentifié par rôle, via les storageState
 * produits par auth.setup.ts (cookies httpOnly access/refresh token).
 *
 * Usage :
 *   test('...', async ({ auth }) => {
 *     const { page } = await auth('manager')
 *     await page.goto('/dashboard')
 *   })
 */
type AuthFixtures = {
  auth: (role: AuthRole) => Promise<{ page: import('@playwright/test').Page }>
}

export const test = base.extend<AuthFixtures>({
  auth: async ({ browser }, use) => {
    const open = async (role: AuthRole) => {
      const ctx = await browser.newContext({
        storageState: `e2e/.auth/${role}.json`,
      })
      const page = await ctx.newPage()
      return { page }
    }
    await use(open)
  },
})

export { expect }
