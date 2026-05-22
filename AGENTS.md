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

### Before starting

1. Locate the matching Epic/Story in `docs/EPICS.md`. No match → confirm with user (new story, new epic, or out-of-scope) before proceeding.
2. If domain rule files conflict with each other or with the request → flag to user and await resolution.
3. Request requires structural decisions → discuss ADR (technical) or DDR (UI/UX) with user before starting. Create only when there are genuine alternatives to document.

### Implementation checklist

Follow in order for any feature or significant change:

1. **ARCHITECTURE.md** — update before writing any new migration (new table, column, index, relation).
2. **DESIGN.md** — add new component to catalogue before implementing it.
3. **ADR / DDR** — create in `docs/adr/` if genuine structural trade-offs exist.
4. Migration: `cargo sqlx migrate run`.
5. Offline query data: `cargo sqlx prepare` → commit `backend/.sqlx/`.
6. Backend: domain → db → api.
7. Frontend.
8. `graphify update .`
9. **EPICS.md** — mark story `✅` and update epic status **only when backend + frontend are both on the real API**. Never anticipate.
10. **LEGAL.md** — if the story covers a legal obligation, update coverage (⚠️/❌ → ✅) in the same pass.
11. **PROGRESS.md** — dated entry with Epic/Story ref (started / completed / abandoned).

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
| Backlog / Epics | `.agents/rules/doc-epics.instructions.md` |
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
