# Backend — Rust Instructions

> Project-wide instructions live in the root `AGENTS.md` (aliased as `CLAUDE.md`). Matching `.claude/rules/` are auto-injected by the IDE when editing backend files. This file covers Rust specifics not covered by a rule.

## Layering

- Zero business logic in handlers (`api/`) — logic lives in `domain/`.
- Zero SQL in `domain/` — DB access in `db/` repositories only.
- `pub(crate)` by default. No `unwrap()` outside tests.
- `Decimal` for all financial amounts — never `f64`.
- Errors: custom `AppError` with i18n code (`AppError::validation_code("errors.x.y")`), never raw strings.

## Migrations & SQLx

- New table → always update `docs/ARCHITECTURE.md` before writing the migration. Create an ADR only when there are genuine structural design trade-offs with real alternatives. Simple tables (lookups, enums, standard joins) → no ADR needed.
- `SQLX_OFFLINE=true` in CI and Docker — `backend/.sqlx/` is committed. After any query change, run `cargo sqlx prepare` and commit the result.
- Migration baseline is `backend/migrations/0001_init.sql` (squashed). Any schema evolution is a new incremental migration — never edit the baseline.

## Type discipline

- Trace every struct field crossing a layer boundary (DB row → domain → API DTO) back to its source of truth and match the type exactly. A numeric cast at a boundary is a red flag — fix the source, not the symptom.
- Typed SQL enums: implement a Rust enum deriving `sqlx::Type`, never cast `::text`.
- Soft-delete filter is mandatory on every query against a soft-deletable table.
