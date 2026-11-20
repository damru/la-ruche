# E2E — La Ruche (Playwright)

Socle de tests end-to-end du frontend : login par rôle, parcours socle,
actions utilisateur (Phase 2), états réseau (DDR-005), garde-fous (DDR-006).

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

⚠️ **ARM64 / Raspberry Pi** : `npx playwright install` ne fournit pas de
chromium complet ARM64 — seul `chromium-headless-shell` est disponible
(suffisant pour ces tests non visuels). Les runs complets multi-navigateurs
restent sur la CI x86.

## Projets

| Projet | Périmètre | Backend requis |
|---|---|---|
| `setup` | Login par rôle via l'API → `e2e/.auth/<role>.json` | oui |
| `chromium` | Socle (routes × rôles, axe), actions utilisateur, garde-fous | oui |
| `mock` | DDR-005 — états réseau par interception `page.route` | non |

`chromium` ignore `tests/reseau/**` ; `mock` ne matche que lui :
`--project=mock` ne démarre que Vite.

## Conventions Phase 2 (actions)

### Données
- Entités de test créées **via l'API** avec le request context de la page
  (`e2e/data/api.ts` : apiGet/apiPost/apiPut/apiDelete) — les cookies httpOnly
  du storageState portent le rôle, les permissions réelles du backend
  s'appliquent. Aucune entité n'est créée par l'UI en setup.
- Identifiants uniques par run et worker : `e2eName(testInfo, prefix)` →
  `e2e-w<worker>-<runid>-<prefix>`. Aucune dépendance d'ordre entre tests.
- Ne pas compter sur des données seedées fixes pour les assertions de contenu.

### Effets UI (DDR-006)
- Feedback inline via `CFormActions` (`e2e/helpers-feedback.ts` :
  `expectFeedback` valid/err, `expectNoFeedback`, `expectSubmitPending`,
  `expectSubmitClosing`, `expectAutoClose`). Le feedback s'auto-efface après
  3 s : les assertions se posent immédiatement après la soumission.
- `CBtn` de type submit est désactivé automatiquement quand le formulaire est
  invalide (`disableWhenFormInvalid` par défaut) — spinner ≠ disabled.
- Le garde-fou statique `tests/archi/ddr006-garde-fou.spec.ts` interdit tout
  `CFormFeedback` utilisé hors de `CFormActions`.

### Pièges connus
- La fixture `auth(rôle)` crée son **propre contexte/page** : `page.route` et
  `page.on` doivent viser la page du helper (`page_o.page`), pas la fixture
  `page` du test — sinon ni interception ni écoute réseau.
- `hasText` = sous-chaîne : utiliser `cardByTitleExact` pour distinguer `X` de
  `X v2`.
- Les modales se rendent dans `.c-modal__content` ; la topbar est un
  `div.c-topbar` (pas un `header`).

## Captures visuelles (Phase 3)

- Projet `visual` : capture des pages stables du layout principal →
  `e2e/visual/<page>--<1440|375>--<light|dark>.png`, committées dans le repo.
- **À générer uniquement par le job CI `visual-preview`** (x86) : sur chaque
  PR touchant `frontend/**`, il recapture et committe les changements →
  l'avant/après s'affiche nativement dans l'onglet Files Changed de la PR
  (vue Swipe). Ne jamais lancer `--project=visual` en local (ARM64 : rendu
  différent, PNGs inexploitables comme références).
- Déterminisme : horloge figée (`page.clock.install`) + thème forcé via
  localStorage + `reducedMotion` — tout élément qui rend une date relative,
  un compteur ou un ID aléatoire doit être figé, sinon churn visuel fantôme.
- Règles complètes : `.claude/rules/visual-regression.instructions.md`.

## Notes CI

- Le job `e2e-frontend` ne lance **pas rustfs** : S3 est inerte en Phase 0
  (tests lecture seule). À ajouter avant les tests mutateurs de Phase 2.
- Les traces uploadées on-failure peuvent contenir les cookies httpOnly —
  jetons dédiés e2e (JWT_SECRET, DB éphémère), inexploitables hors du run.

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
- Phase 0 = lecture seule + login. Phase 2 ajoute les tests mutateurs (création
  via API, reset encadré) et les garde-fous archi.
