import { defineConfig, devices } from '@playwright/test'

/**
 * La Ruche — E2E (Phase 0 : socle).
 *
 * Le webServer démarre Vite (port 5173) dont le proxy /api pointe vers le
 * backend de dev (:8080). Prérequis local : `make setup && make dev`
 * (postgres + rustfs + backend, DEMO_MODE=true → seed demo.sql).
 */
export default defineConfig({
  testDir: './e2e',
  testMatch: '**/*.spec.ts',
  fullyParallel: true,
  retries: process.env.CI ? 1 : 0,
  // Pi 4 Go : 4 workers chromium + Vite dépassent la RAM (OOM → crash Vite
  // en cours de run, ERR_CONNECTION_REFUSED en cascade). 2 workers local.
  workers: process.env.CI ? 2 : 2,
  reporter: process.env.CI ? [['html', { open: 'never' }], ['list']] : [['list']],

  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },

  use: {
    baseURL: 'http://localhost:5173',
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

  // Timeout par test : le parcours login→dashboard→profile→logout charge
  // ~20 requêtes API ; en parallèle sur Pi il dépasse 30 s.
  timeout: 60_000,

  projects: [
    { name: 'setup', testMatch: '**/auth.setup.ts' },
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
      testIgnore: '**/tests/reseau/**',
    },
    {
      // DDR-005 : états réseau via interception, sans backend ni postgres.
      name: 'mock',
      use: { ...devices['Desktop Chrome'] },
      testMatch: '**/tests/reseau/**',
    },
  ],
})
