---
description: 'Accounting immutability rules — apply whenever creating or modifying an entity that carries financial amounts, legal dates, or lifecycle statuses (DB migrations, Rust structs/handlers/domain, Vue forms/modals/cards).'
applyTo: 'backend/**/*.rs,backend/migrations/**/*.sql,frontend/src/**/*.{vue,ts}'
---

# Accounting Immutability (ADR-009)

> Full specification: `docs/adr/ADR-009-accounting-immutability.md`  
> Legal basis: `docs/LEGAL.md §11`

---

## Step 1 — Does this entity carry financial or legal traceability data?

**Signals — answer yes if any of these apply:**
- stores an `amount`, `cost`, or financial balance
- stores a date that triggers a legal obligation (`declared_at`, `transmitted_at`, `closed_at`, `due_date`…)
- has a `status` enum that reflects a lifecycle transition (open → closed, draft → issued…)
- is referenced by another entity that does any of the above

**If no → no further action required.**  
**If yes → proceed to Step 2.**

---

## Step 2 — Which pattern applies?

| Pattern | When | Mechanism |
|---|---|---|
| **A — Freeze on terminal state** | Entity stays in place; some fields become immutable after a status transition | Guard `domain::immutability::is_mutable()` / `is_deletable()` called before any write; dedicated transition endpoint |
| **B — Close + recreate** | Modifying in place is semantically wrong (amendment, rectifying budget) | `predecessor_id` or `parent_budget_id` column; `POST /{id}/terminate` or `POST /{id}/amend` endpoint |
| **C — Append-only** | Correction = new write; original row stays intact | No `UPDATE`/`DELETE`; new row referencing the previous one |

**Currently covered entities:**

| Entity | Pattern | Frozen from |
|---|---|---|
| `levy_call` | A | `status = 'issued'` |
| `vote` / `resolution` | A | AG `ongoing` / `closed` |
| `task.completion_cost`, `task.closed_at` | A | `status = 'closed'` |
| `claim.transmitted_at`, `claim.reimbursement_*` | A | Fields set |
| `maintenance_log` (source `automatic`) | A | Always |
| `vendor_contract` | B | Active — amend via `predecessor_id` |
| `budget` / `budget_line` | B | Approved — rectify via `parent_budget_id` |
| `maintenance_log` (source `manual`) | C | Always |
| Correction of an issued `levy_call` | C | Credit note (negative amount row) |

---

## Step 3 — Does this entity need point-in-time queries?

**Signal:** a derived value (levy call, allocated amount) is calculated **from** this entity and must be reproducible at a past date.

If yes → add `valid_from DATE NOT NULL` and `valid_to DATE` columns.  
Reference implementation: `unit_allocation_key` (ADR-006), `budget_line` (ADR-009).

If no → Pattern A or B suffices.

---

## Backend rules (`.rs`, `migrations/*.sql`)

- **No generic `PUT`/`PATCH` on frozen fields.** Use a dedicated transition endpoint (`POST /{id}/close`, `POST /{id}/issue`, `POST /{id}/transmit`, `POST /{id}/terminate`).
- **Every transition endpoint must call `domain::immutability::is_mutable()` before any DB write.**
- **Every `DELETE` handler on a critical entity must call `domain::immutability::is_deletable()` before executing.**
- Financial mutations must write to `audit_log` via `db::audit_log::record()` with `old_values` / `new_values` as JSONB containing **only the changed fields**.
- Log: `UPDATE` on financial/traceability fields, `DELETE` attempts, status transitions. Do **not** log `INSERT` or non-financial field updates.
- `audit_log` rows are append-only — never delete or update them.

---

## Frontend rules (`.vue`, `.ts`)

- **Pattern A — frozen fields:** use `v-if` (not `disabled`) to hide Modifier / Supprimer once the terminal state is reached.
- **Pattern A — readonly display:** render immutable fields as `<span>`, not `<input readonly>`. Add a discrete `label-sm .tone--muted` label « Donnée comptable — non modifiable ».
- **Pattern A — frozen indicator:** display `CIconLock` with `.tone--muted` in the card or detail header.
- **Pattern B — amend workflow:** replace « Modifier » with « Créer un avenant » (`ghost` variant, not destructive). Pre-fill the new form and close the old entity in one server call.
- **Never call a generic `PUT`/`PATCH`** on a frozen entity — use the dedicated transition endpoint.
- **Never include frozen fields in a form payload** once they are set.
