---
description: 'Rules for modifying docs/DESIGN.md, DDR files, and frontend component catalogue.'
applyTo: 'docs/DESIGN.md,docs/ddr/**/*.md,frontend/src/**/*.vue,frontend/src/**/*.ts,frontend/src/**/*.css'
---

# Design System Docs Rules

Source of truth: `docs/DESIGN.md`. References: DDR-001 to DDR-006.

## When to modify

Only after a new DDR is validated. Edits: inline, minimal, in existing section. No new section without user approval.

## Mandatory

- No raw values (`px`, `hex`, `rem`) in `class`/`style` if a CSS token exists.
- New `atom`/`molecule`/`organism` → add to catalogue (`docs/DESIGN.md §2`) before use.
- Interactive primitives → wrap in `C*` component. Never import `radix-vue` directly in a feature view.
- `.tone--*` classes are pure CSS utilities — never redefine locally.
- No `<style>` block in feature views — use Tailwind utilities or catalogue components.
- `CFormFeedback` → always in `#footer` slot of `CModal`, never in body.
- No external references (ADR, ticket numbers) in `docs/DESIGN.md` body — DDR refs only.

## Do not put here

Backend/DB rules → `docs/ARCHITECTURE.md`. Legal → `docs/LEGAL.md`. Product workflows → `docs/PRD.md`.
