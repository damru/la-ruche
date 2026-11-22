import { test, expect } from '../fixtures/auth'
import { frText } from '../helpers-i18n'
import { attachCollectors } from '../helpers-net'
import { APP_ROUTES, allowedAccounts, type AuthRole, type MatrixEntry } from '../data/access-matrix'

test.describe('Smoke — routes autorisées par rôle', () => {
  for (const route of APP_ROUTES) {
    for (const role of allowedAccounts(route.roles)) {
      test(`${route.path} rend pour ${role}`, async ({ auth }) => {
        const { page, close } = await auth(role)
        const net = attachCollectors(page)

        await page.goto(route.path)

        await page.waitForLoadState('networkidle')

        await expect(page.locator('.view-header').first()).toBeVisible()
        if (route.titleKey) {
          await expect(page.getByText(frText(route.titleKey)).first()).toBeVisible()
        }

        await expect(page.locator('.c-page-error')).toHaveCount(0)

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

    const denied = ALL.filter((r) => !allowedAccounts(route.roles).includes(r))
    for (const role of denied) {
      test(`${route.path} → /403 pour ${role}`, async ({ auth }) => {
        const { page, close } = await auth(role)
        const net = attachCollectors(page)

        await page.goto(route.path)
        await page.waitForLoadState('networkidle')

        if ((route.deniedBehavior ?? 'route403') === 'pageError') {

          await expect(page.locator('.c-page-error')).toBeVisible()
          expect(net.apiFailures, `API en erreur sur ${route.path}`).toEqual([
            expect.stringMatching(/^403 /),
          ])
          await close()
          return
        }

        await expect(page).toHaveURL(/\/403/)
        await expect(page.getByText(frText('errors.forbidden')).first()).toBeVisible()

        expect(net.apiFailures, `API en erreur sur ${route.path} (interdit)`).toEqual([])
        await close()
      })
    }
  }
})

export type { MatrixEntry }
