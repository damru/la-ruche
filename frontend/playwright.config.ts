import { defineConfig, devices } from '@playwright/test'

/**
 * La Ruche — E2E (Phase 0: socle).
 *
 * The webServer starts Vite (port 5173) whose /api proxy points to the
 * dev backend (:8080). Local prerequisite: `make setup && make dev`
 * (postgres + rustfs + backend, DEMO_MODE=true -> demo.sql seed).
 */
export default defineConfig({
  testDir: './e2e',
  testMatch: '**/*.spec.ts',
  fullyParallel: true,
  retries: process.env.CI ? 1 : 0,
  // 4 GB Pi: 4 chromium workers + Vite exceed RAM (OOM -> mid-run Vite
  // crash, cascading ERR_CONNECTION_REFUSED). 2 workers local.
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
    // The login->dashboard flow loads ~16 API calls on mount;
    // on a Raspberry Pi it exceeds 5 s. Marges sized for the hardware.
    timeout: 15_000,
  },

  // Per-test timeout: the login->dashboard->profile->logout flow loads
  // ~20 API calls; in parallel on a Pi it exceeds 30 s.
  timeout: 60_000,

  projects: [
    { name: 'setup', testMatch: '**/auth.setup.ts' },
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
      testIgnore: ['**/tests/reseau/**', '**/tests/visual/**'],
    },
    {
      // DDR-005: network states via interception, no backend, no postgres.
      name: 'mock',
      use: { ...devices['Desktop Chrome'] },
      testMatch: '**/tests/reseau/**',
    },
    {
      // Phase 3: visual preview captures — repo-committed PNGs, CI x86 only.
      // Subpixel text positioning renders identical glyphs with different AA
      // depending on fractional layout state — the classic source of
      // screenshot flakiness. Disable it (and LCD text) for stable pixels.
      name: 'visual',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: {
          args: ['--disable-font-subpixel-positioning', '--disable-lcd-text'],
        },
      },
      dependencies: ['setup'],
      testMatch: '**/tests/visual/**',
      testIgnore: ['**/pages-capture-empty.spec.ts'],
    },
    {
      // Empty-state pass: runs after the seeded captures, on a truncated
      // database (accounts and building survive). Serial + one worker so the
      // reset never races a capture.
      name: 'visual-empty',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: {
          args: ['--disable-font-subpixel-positioning', '--disable-lcd-text'],
        },
      },
      dependencies: ['setup'],
      testMatch: '**/pages-capture-empty.spec.ts',
      workers: 1,
    },
  ],
})
