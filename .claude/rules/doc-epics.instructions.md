---
description: 'Rules for modifying docs/EPICS.md.'
applyTo: 'docs/EPICS.md'
---

# Epics & Stories Rules

Source of truth for backlog: `docs/EPICS.md`. Legal coverage table: `docs/LEGAL.md` only.

## Lifecycle

`new`🆕 → `in progress` ⏳ → `delivered` ✅ (backend + frontend on real API) | `canceled ❌` → `out of scope` ❌ (moved to "Fonctionnalités hors scope" with justification).

## When to modify

| Trigger | Action |
|---|---|
| Work started on a story | Add `⏳` |
| Story delivered (backend + frontend on real API) | Add `✅`, update epic status, update "État code actuel", update the story spec's `Statut actuel` line in `docs/specs/` (e.g. `IMPLÉMENTÉ (backend + frontend)`) |
| New story from PRD, ADR or legal obligation | Add story with explicit ref (`LEGAL.md §X.Y` / `PRD US-XX` / `ADR-XXX`) and reopen the epic if necessary |
| New epic decided | Add with status, PRD/ADR/DDR ref, and at least one story |
| Story out of scope | Move to "Fonctionnalités hors scope" with justification |
| New or amended legal reference in `LEGAL.md` | Create or update the linked story pointing to `LEGAL.md §X.Y` |

## Mandatory

- Never mark `✅` unless backend + frontend are both on the real API.
- Never include implementation details in a story (no filenames, DB schemas, function names, endpoints) — those belong in ADRs and code.
- Every story must reference a PRD, ADR, or `LEGAL.md §X.Y`.
- Never change an epic status without checking each of its stories.
- Never recreate a legal coverage table here — that lives exclusively in `docs/LEGAL.md`.

## Story format

```
* `EP-XXX-SYY` [✅] — Action verb + functional scope (LEGAL.md §X.Y)
```

## Cross-sync on delivery

When a `✅` story covers a legal obligation → update `docs/LEGAL.md` coverage (⚠️/❌ → ✅) in the same pass. Never anticipate: coverage ✅ only after effective delivery.

## Do not put here

Implementation details → ADRs and code. Legal coverage table → `docs/LEGAL.md`. Feature detail → `docs/product/PRD - *.md`.
