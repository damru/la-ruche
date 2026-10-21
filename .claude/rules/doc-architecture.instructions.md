---
description: 'Rules for modifying docs/ARCHITECTURE.md and related ADR files.'
applyTo: 'docs/ARCHITECTURE.md,docs/adr/**/*.md'
---

# Architecture Docs Rules

Source of truth: `docs/ARCHITECTURE.md`. References: ADR-001 to ADR-012.

## When to modify

Only modify if change is **proven in code**: new migration, new `src/` module, stack change, or ADR created/amended. Never anticipate — use "planned" marker if needed.

## Mandatory

- New table / handler / `domain/` module / frontend store → update `docs/ARCHITECTURE.md` same commit.
- Table/column/Rust type names → English identifiers from code only.
- "Planned, not yet implemented" entities → must reference an ADR or Epic. Do not code them without one.
- Schéma DB invariants are contracts — any change requires sync update here.
- Never delete an invariant without verifying it's no longer enforced in code.
- No external references (ADR, DDR, ticket numbers) in `docs/ARCHITECTURE.md` body — refs only.

## Do not put here

Product rules → `docs/PRD.md`. UX/tokens → `docs/DESIGN.md`. Legal obligations → `docs/LEGAL.md`.
