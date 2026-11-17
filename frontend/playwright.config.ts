import { defineConfig, devices } from '@playwright/test'

/**
 * La Ruche — E2E (Phase 0 : socle).
 *
 * Le webServer démarre Vite (port 5173) dont le proxy /api pointe vers le
 * backend de dev (:8080). Prérequis local : `make setup && make dev`
 * (postgres + rustfs + backend, DEMO_MODE=true → seed demo.sql).
 */
export default defineConfig({
  testDir: './e2e/tests',
  fullyParallel: true,
  // Phase 0 : le socle n'a pas besoin de retries ; 1 retry en CI uniquement.
  retries: process.env.CI ? 1 : 0,
  workers: process.env.CI ? 2 : 4,
  reporter: process.env.CI ? [['html', { open: 'never' }], ['list']] : [['list']],

  use: {
    baseURL: 'http://localhost:5173',
    // Hiérarchie de timeouts : action (10s) > API_TIMEOUT_MS du client axios (5s).
    actionTimeout: 10_000,
    navigationTimeout: 15_000,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },

  expect: {
    // Le parcours login→dashboard charge ~16 requêtes API au montage ;
    // sur Raspberry Pi il dépasse 5 s. Marges adaptées au matériel.
    timeout: 15_000,
  },

  projects: [
    // Setup : connexion par rôle via l'API (cookies httpOnly) → storageState.
    { name: 'setup', testMatch: /auth\.setup\.ts/ },
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
    },
  ],
})
