# La Ruche — Agent Instructions

## Project

Open-source webapp for French co-ownership management (syndic bénévole).
Monorepo: `backend/` (Rust) + `frontend/` (Vue 3).

Domain-specific rules in `.agents/rules/` are automatically injected into context by the IDE when editing files in the matching domain. Do not re-read them manually.

---

## Commands

```bash
# Backend
cargo build                        # compile
cargo test --lib --tests           # run tests
cargo clippy -- -D warnings        # lint (must pass clean)
cargo fmt --check                  # format check

# Frontend
npm --prefix frontend install      # install deps
npm --prefix frontend run build    # build
npm --prefix frontend run lint     # lint

# DB (local dev)
docker compose up -d postgres      # start postgres
cargo sqlx migrate run             # apply migrations
cargo sqlx prepare                 # regenerate .sqlx/ (offline mode)
```

---

## Language

- **Code + comments** → English only.
- **Docs** (PRD, DESIGN, ARCHITECTURE, PROGRESS, ADR, DDR) → French. Code identifiers stay English inside docs.

---

## Backend (Rust + Axum + SQLx)

- Zero business logic in handlers (`api/`) — logic lives in `domain/`.
- Zero SQL in `domain/` — DB access in `db/` repositories only.
- `pub(crate)` by default. No `unwrap()` outside tests.
- `Decimal` for all financial amounts — never `f64`.
- New table → always update `docs/ARCHITECTURE.md` before migration. Create an ADR only when there are **genuine structural design trade-offs** with real alternatives (e.g. schema design, storage strategy). Simple tables (lookups, enums, standard joins) → no ADR needed.
- `SQLX_OFFLINE=true` in CI and Docker — `backend/.sqlx/` committed.
- Errors: custom `AppError` with i18n code (`AppError::validation_code("errors.x.y")`), never raw strings.

## Frontend (Vue 3 + TypeScript + Tailwind v4)

- Composition API only. No Options API.
- No raw values (`px`, `hex`, `rem`) in `class`/`style` — use Tailwind utilities or CSS tokens.
- No `<style>` block in feature views — Tailwind utilities or catalogue components only.
- `CFormFeedback` in `#footer` slot of `CModal` only, never in body.
- No direct `radix-vue` import in feature views — use `C*` wrappers.
- New component → add to catalogue in `docs/DESIGN.md §2` before use.

---

## Workflow

- Request matches existing PRD or Epic → confirm with user before starting.
- ADR for structural technical decisions with real alternatives. DDR for structural UI/UX decisions.
- Create ADR/DDR only when there are genuine alternatives to document.
- If domain rule files conflict with each other or with user requests, flag the conflict to the user and await resolution before proceeding.

## Feature Implementation Checklist

Follow these steps **in order** for any new feature or significant change:

1. Update `docs/PRD.md` / `docs/DESIGN.md` / `docs/ARCHITECTURE.md` **before** coding.
2. If structural DB trade-offs exist, create an ADR in `docs/adr/` before writing the migration.
3. Write and apply the migration (`cargo sqlx migrate run`).
4. Regenerate offline query data (`cargo sqlx prepare`) and commit `backend/.sqlx/`.
5. Implement backend changes (domain → db → api layers).
6. Implement frontend changes.
7. Run `graphify update .` to keep the knowledge graph current.
8. Add a dated entry to `docs/PROGRESS.md` with Epic/Story ref (started / completed / abandoned).

---

## Accounting immutability (ADR-009)

Full rules → `.agents/rules/accounting-immutability.instructions.md`

Short version: financial amounts, legal dates, lifecycle statuses → freeze on terminal state (Pattern A), close+recreate for amendments (Pattern B), or append-only (Pattern C). Never generic `PUT`/`PATCH` on frozen fields.

---

## Domain rules index

| Domain | Rule file |
|---|---|
| Architecture / backend docs | `.agents/rules/doc-architecture.instructions.md` |
| Design system / frontend docs | `.agents/rules/doc-design.instructions.md` |
| Legal / compliance | `.agents/rules/doc-legal.instructions.md` |
| Product / PRD | `.agents/rules/doc-prd.instructions.md` |
| Accounting immutability | `.agents/rules/accounting-immutability.instructions.md` |
| Rust conventions | `.agents/rules/rust.instructions.md` |
| Vue.js conventions | `.agents/rules/vue.instructions.md` |
| Tailwind CSS v4 | `.agents/rules/tailwind-v4-vite.instructions.md` |
| Knowledge graph (MCP) | `.agents/rules/code-review-graph.instructions.md` |

---

## Security

- JWT: `httpOnly` + `Secure` + `SameSite=Strict`. Refresh token hashed SHA-256, never in plain text.
- All queries filter on `building_id` — no cross-tenant leaks.
- 403 (not 404) on protected routes.
- Files: private S3 bucket, pre-signed URLs only.
- Never commit secrets or `.env` files.

---

## graphify

Knowledge graph at `graphify-out/`.

- Architecture/codebase questions → read `graphify-out/GRAPH_REPORT.md` first.
- After modifying code files → run `graphify update .` to keep the graph current (also step 7 of the Feature Implementation Checklist).
