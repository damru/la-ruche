import { test, expect } from '../fixtures/auth'

test.describe('Focus — ordre et visibilité', () => {
  test('login : ordre de focus logique email → mot de passe → actions', async ({ page }) => {
    await page.goto('/login')
    await page.locator('#email').focus()

    await page.keyboard.press('Tab')
    await expect(page.locator('#password')).toBeFocused()

    await page.keyboard.press('Tab')
    const focused = page.evaluate(() => document.activeElement?.tagName ?? '')
    expect(['BUTTON', 'A', 'INPUT']).toContain(await focused)
  })

  test('focus visible : Tab pose un outline sur un champ du login', async ({ page }) => {
    await page.goto('/login')
    await page.keyboard.press('Tab')
    await page.keyboard.press('Tab')

    const outline = await page.evaluate(() => {
      const el = document.activeElement as HTMLElement | null
      if (!el) return 'none'
      const s = getComputedStyle(el)
      return `${s.outlineWidth} ${s.outlineStyle} | boxShadow=${s.boxShadow !== 'none'}`
    })

    const visible = !outline.startsWith('0px none') || outline.includes('boxShadow=true')
    expect(visible, `focus non visible sur l'élément : ${outline}`).toBe(true)
  })
})

test.describe('Cibles tactiles (WCAG 2.5.8 AA ≥ 24 px)', () => {
  test('boutons du login ≥ 24 px', async ({ page }) => {
    await page.goto('/login')

    await expect(page.locator('#email')).toBeVisible()
    const buttons = page.locator('button:visible')
    const count = await buttons.count()
    expect(count).toBeGreaterThan(0)

    for (let i = 0; i < count; i++) {
      const box = await buttons.nth(i).boundingBox()
      if (!box) continue
      expect(
        Math.min(box.width, box.height),
        `cible tactile < 24 px sur le bouton #${i} du login`,
      ).toBeGreaterThanOrEqual(24)
    }
  })

  test('liens de navigation (sidebar desktop) ≥ 24 px de haut', async ({ auth }) => {
    const { page, close } = await auth('manager')
    await page.goto('/dashboard')
    await page.waitForLoadState('networkidle')

    const navLinks = page.locator('.c-sidebar__item:visible')

    await expect(navLinks.first()).toBeVisible()
    const count = await navLinks.count()
    expect(count).toBeGreaterThan(0)

    const tooSmall: string[] = []
    for (let i = 0; i < count; i++) {
      const box = await navLinks.nth(i).boundingBox()
      const label = (await navLinks.nth(i).textContent())?.trim() ?? `#${i}`
      if (box && box.height < 24) tooSmall.push(`${label} (${box.height}px)`)
    }
    expect(tooSmall, `liens de nav < 24 px de haut`).toEqual([])
    await close()
  })
})
