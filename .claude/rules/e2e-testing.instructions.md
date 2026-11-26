---
description: 'Conventions for frontend/e2e/** (Playwright): API-created test data, feedback helpers, known traps. Authoritative details: frontend/e2e/README.md.'
applyTo: 'frontend/e2e/**'
---

# E2E testing (Playwright)

The authoritative conventions live in `frontend/e2e/README.md` — read it before
writing or reviewing any e2e test. Non-negotiables:

- **Test data via API, never via UI or fixed seeds**: create entities with the
  page's request context (`e2e/data/api.ts`: apiGet/apiPost/apiPut/apiDelete);
  storageState cookies carry the role, real backend permissions apply. Unique
  names per run+worker via `e2eName(testInfo, base)`. No cross-test order
  dependencies; never assert on fixed seeded content.
- **Auth**: use the `auth(role)` fixture (`e2e/fixtures/auth.ts`) — login happens
  once in `auth.setup.ts` (API → storageState), never inside a test.
- **Form feedback (DDR-006)**: use `e2e/helpers-feedback.ts` assertions
  (`expectFeedback`, `expectNoFeedback`, …); feedback auto-clears after 3 s, so
  assert immediately after submit.
- **Network states (DDR-005)**: intercept with `page.route` on the page created
  by the helper — the `auth` fixture creates its OWN page; `page.route`/`page.on`
  on the test's bare `page` fixture silently does nothing (known trap).
- **Access matrix**: route × role expectations come from
  `e2e/data/access-matrix.ts` — add new routes there; the smoke and capture
  suites derive from it. Do not duplicate role logic in specs.
- **Timeouts**: margins are sized for a Raspberry Pi (config comment) — do not
  tighten them to make a local run look faster.
- **Determinism**: any UI element rendering a relative date, counter or random
  ID must be frozen (cf. `tests/visual/pages-capture.spec.ts` for the clock
  pattern) — flaky-by-time tests are bugs.
- **No new suite for a new check**: add the case to an existing spec — static
  repository guards go into `tests/archi/ddr006-garde-fou.spec.ts` (whatever
  their topic), cross-page consistency checks into
  `tests/consistance-matrix.spec.ts`, entity behaviour into the entity's spec.
  A new `.spec.ts` file must introduce a new entity or a new mechanism, never a
  single new case.
- **CSS regressions: assert computed styles, never class attributes.** Classes
  can be correct and still lose the cascade — unlayered `base.css` rules (e.g.
  `a { color: … }`) beat every Tailwind utility, so a heading link renders in
  the link colour while every class assertion and axe AA check passes. Pattern:
  read a probe element's `getComputedStyle(el).color` for the expected token,
  then compare it with the real element's computed colour on each matrix route.
- Visual captures (`tests/visual/**`) have their own rule:
  `visual-regression.instructions.md`.
