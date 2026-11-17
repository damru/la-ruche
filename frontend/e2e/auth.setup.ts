import { test as setup, expect } from '@playwright/test'
import { ACCOUNTS, DEV_PASSWORD, type AuthRole } from './fixtures/accounts'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

/**
 * Setup d'authentification : login par API pour chaque rôle.
 *
 * Mécanisme auth réel (ADR-002) : POST /api/v1/auth/login renvoie le profil
 * sans token dans le corps ; le JWT est porté par les cookies httpOnly
 * access_token + refresh_token. storageState capture ces cookies — aucune
 * injection localStorage.
 *
 * En local (http), le backend ne pose pas le flag Secure
 * (PUBLIC_URL ne commence pas par https) — les cookies sont acceptables.
 */

const stateDir = path.join(path.dirname(fileURLToPath(import.meta.url)), '.auth')

for (const role of Object.keys(ACCOUNTS) as AuthRole[]) {
  setup(`authentifié en tant que ${role}`, async ({ request }) => {
    const res = await request.post('/api/v1/auth/login', {
      data: { email: ACCOUNTS[role], password: DEV_PASSWORD },
    })
    expect(res.status()).toBe(200)

    fs.mkdirSync(stateDir, { recursive: true })
    await request.storageState({ path: path.join(stateDir, `${role}.json`) })
  })
}
