---
description: 'Rules for modifying docs/LEGAL.md — French co-ownership law obligations and coverage tracking.'
applyTo: 'docs/LEGAL.md'
---

# Legal Docs Rules

Source of truth: `docs/LEGAL.md`. Scope: French co-ownership law, RGPD, legal accounting, syndic obligations.

## When to modify

Only when a law/decree/jurisprudence changes, or coverage status changes after an **actual delivery**.

## Mandatory

- New or modified legal obligation → add to `docs/LEGAL.md` **before** any backlog item or ADR.
- Every legal reference must cite the official text: Légifrance LEGIARTI ID + version + effective date.
- Never interpret the law — quote official texts verbatim.
- Coverage gap (⚠️ partial / ❌ uncovered) → must point to an explicit Epic or story in `docs/EPICS.md`.
- ✅ coverage → only after actual delivery, never after planning.
- Never delete an obligation — archive with "repealed by [text] on [date]" if necessary.
- No external references (ADR, ticket numbers) in `docs/LEGAL.md` body — refs only.

## Do not put here

Technical implementation → `docs/ARCHITECTURE.md` + ADR. Product workflows → `docs/PRD.md`. UX → `docs/DESIGN.md`.
