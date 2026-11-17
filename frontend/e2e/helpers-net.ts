import type { Page } from '@playwright/test'

const API_PREFIX = '/api/v1'

/** 401 légitimes des sondes de session, émises avant authentification. */
export const LEGITIMATE_401 = ['/api/v1/auth/me', '/api/v1/auth/refresh']

export type NetCollectors = {
  consoleErrors: string[]
  apiFailures: string[]
}

/**
 * Attache les collecteurs console/réponse AVANT la navigation (sinon les
 * erreurs du chargement initial sont manquées).
 *
 * Filtrage du bruit légitime :
 * - 401 des sondes /auth/me et /auth/refresh pendant restoreSession
 * - messages console "Failed to load resource" redondants avec l'API
 *   (le statut HTTP est déjà capturé côté response)
 */
export function attachCollectors(page: Page): NetCollectors {
  const consoleErrors: string[] = []
  const apiFailures: string[] = []
  page.on('console', (msg) => {
    if (msg.type() !== 'error') return
    // Chromium émet un console.error générique par ressource 4xx/5xx ;
    // l'échec API est déjà tracé par le listener 'response'.
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
