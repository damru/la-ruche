---
description: 'Rules for modifying docs/PRD.md and docs/product/PRD - *.md feature files.'
applyTo: 'docs/PRD.md,docs/product/**/*.md'
---

# PRD Docs Rules

Source of truth: `docs/PRD.md` (master) + `docs/product/PRD - {feature}.md` (per-feature detail).

## When to modify

`docs/PRD.md`: only to add/remove a feature, update product vision, or fix transversal info (roles, legal constraints, architecture summary). Feature detail lives in its own PRD file.

## Mandatory

- `docs/PRD.md` body contains no external technical references (ADR, DDR, ticket numbers).
- Abandoned features → send to **Cimetière** section with justification. Never delete.
- Before starting any work: check if request matches an existing PRD or Epic. If yes → confirm before proceeding.
- Feature PRDs with legal constraints must reference `docs/LEGAL.md`.
- Link functional decisions to technical constraints via ADR/DDR refs — in feature PRDs, not in `docs/PRD.md`.
- No external references (ADR, DDR, ticket numbers) in `docs/PRD.md` or `docs/product/**/*.md` body — refs only.

## Do not put here

Technical implementation → `docs/ARCHITECTURE.md`. Design/components → `docs/DESIGN.md`. Legal text → `docs/LEGAL.md`.
