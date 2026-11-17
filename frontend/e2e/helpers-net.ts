import type { Page } from '@playwright/test'

const API_PREFIX = '/api/v1'

export const LEGITIMATE_401 = ['/api/v1/auth/me', '/api/v1/auth/refresh']

export type NetCollectors = {
  consoleErrors: string[]
  apiFailures: string[]
}

export function attachCollectors(page: Page): NetCollectors {
  const consoleErrors: string[] = []
  const apiFailures: string[] = []
  page.on('console', (msg) => {
    if (msg.type() !== 'error') return

    if (msg.text().startsWith('Failed to load resource')) return
    consoleErrors.push(msg.text())
  })
  page.on('response', (res) => {
    const url = new URL(res.url())
    if (!url.pathname.startsWith(API_PREFIX)) return
    if (res.status() < 400) return
    const legit = res.status() === 401 && LEGITIMATE_401.includes(url.pathname)
    if (!legit) {
      apiFailures.push(`${res.status()} ${res.request().method()} ${url.pathname}`)
    }
  })
  return { consoleErrors, apiFailures }
}
