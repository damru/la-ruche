---
description: 'Validity intervals: versioned data uses valid_from/valid_to, never overwritten; consumers snapshot at read/creation time.'
applyTo: 'backend/src/**,frontend/src/**,backend/migrations/**'
---

# Validity intervals (temporal versioning)

Any domain data whose value changes over time with retroactive effect on
documents (allocation keys, budgets, contracts, insurance periods, fee
schedules…) follows the same temporal pattern:

- **Versioned rows, never overwritten**: a change closes the current version
  (`valid_to`) and creates a new one (`valid_from`); history is immutable.
- **Point-in-time reads**: consumers resolve the version in force at a given
  date via `valid_from <= date < valid_to` (`valid_to` null = current).
- **Snapshot at consumption**: documents that must not drift (levy calls,
  decomptes, PV) snapshot the values in force at their creation; they are
  never recomputed retroactively.
- **Consistency invariants**: where a version carries distribution data
  (tantièmes), the sum must stay consistent per building/type; any write
  violating the invariant is rejected.
- Migrations must never retroact on existing snapshots.

Reference implementations: allocation keys (ADR-006, ADR-007), budget lines
(optimistic locking, ADR-012).
