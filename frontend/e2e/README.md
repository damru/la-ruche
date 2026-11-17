# E2E — La Ruche (Playwright)

Socle de tests end-to-end du frontend. Phase 0 : login par rôle + parcours socle.

## Prérequis

1. Stack de dev démarré :
   ```
   make setup          # .env (DEMO_MODE=true, etc.)
   make dev            # postgres + rustfs + backend (migrations + seed auto)
   ```
   (`make dev` ne lance PAS Vite — Playwright démarre Vite lui-même via
   `webServer`. Pour un Vite persistant à côté : `make dev-frontend`.)
   Ou docker compose. Le backend seed la base au démarrage si `DEMO_MODE=true`
   et base vide (`backend/seeds/demo.sql`, comptes de `docs/DEV_ACCOUNTS.md`,
   mot de passe `Demo1234!`).
2. Navigateurs Playwright :
   ```
   npx playwright install chromium
   ```

## Lancer

```
npx playwright test            # tout
npx playwright test --ui       # mode interactif
npx playwright test --headed   # navigateur visible
```

## Structure

```
e2e/
  auth.setup.ts          # login API par rôle → storageState (.auth/<role>.json)
  fixtures/accounts.ts   # comptes seedés
  fixtures/auth.ts       # fixture `auth(role)` → page authentifiée
  tests/                 # specs
```

## Notes

- L'auth passe par **cookies httpOnly** (`access_token`/`refresh_token`) posés
  par `POST /api/v1/auth/login` — capturés par `storageState`, jamais
  d'injection localStorage.
- Le seed est ignoré si la base contient déjà des comptes ; re-seed via
  `POST /api/v1/admin/reset-demo` (efface les données de dev manuel).
- Phase 0 = lecture seule + login. Les tests mutateurs arrivent en Phase 2
  (création via API, reset encadré).
