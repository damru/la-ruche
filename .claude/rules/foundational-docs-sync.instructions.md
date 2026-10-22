---
description: 'Foundational docs (ARCHITECTURE, DESIGN, EPICS, PRD) must be updated in the same change as the code that invalidates them.'
applyTo: '**'
---

# Foundational document sync

The four foundational docs are contracts, not archives:

- `docs/ARCHITECTURE.md` — technical architecture and module descriptions.
- `docs/DESIGN.md` — product identity and design system.
- `docs/EPICS.md` — epic/story backlog and delivery status.
- `docs/PRD.md` — product vision, users, features, constraints.

## Rules

- Any change that invalidates a foundational doc (new table/module/route,
  new design component, story delivered, feature scope change) updates the
  doc **in the same commit**, not in a follow-up.
- Story delivery updates its status in `docs/EPICS.md` **and** the
  `Statut actuel` line of its spec in `docs/specs/EP-XXX-SYY-spec.md`
  (the spec describes the current state of the code: a delivered story is
  marked `IMPLÉMENTÉ (backend + frontend)`, a partial one `PARTIEL` with the
  remaining gaps); a new design-system
  component updates `docs/DESIGN.md`; a schema change updates
  `docs/ARCHITECTURE.md`.
- Structural changes only: keep these docs free of implementation detail;
  detail belongs in `docs/specs/`, ADRs and DDRs.
- Never delete an invariant from a foundational doc without verifying it is
  no longer enforced in code.
- If a change is too large for in-place update, flag the doc with a dated
  "outdated" note and open an issue — never leave silent drift.
