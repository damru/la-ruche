---
description: 'Database conventions: UUID v4, TIMESTAMPTZ, one soft-delete per table, updated_at triggers, single migration baseline.'
applyTo: 'backend/migrations/**,backend/src/**'
---

# PostgreSQL conventions

- Primary keys: UUID v4 generated application-side, type `uuid` in PostgreSQL.
- Timestamps: `TIMESTAMPTZ` only, never `TIMESTAMP` without time zone.
- A table has exactly **one soft-delete mechanism** (`deleted_at` or equivalent); never add a second redundant flag.
- Every mutable table has an `updated_at` trigger; the application never sets `updated_at` manually.
- Single migration baseline: `backend/migrations/0001_init.sql` (squashed). Any schema evolution is a new incremental migration, never a baseline edit.
- Any new table must reflect its invariants in `docs/ARCHITECTURE.md` and respect the ADRs in force (e.g. ADR-006/007/012 depending on the domain).
- When a SQL literal becomes a bound parameter, renumber **all** placeholders contiguously (`$1..$N`): compile-checked queries (`sqlx::query!`) fail on a numbering gap, and runtime queries bind silently out of order — both are bugs introduced by partial renumbering.
- A heuristic backfill must document, in the migration comment, the values it can **never** assign and why (e.g. no linking column for a category) — the next reader must not go looking for the missing line.
