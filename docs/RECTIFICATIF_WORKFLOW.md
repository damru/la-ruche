# Budget Rectificatif Workflow

**Status:** Implemented  
**Related:** ADR-009 §Pattern B (Immutabilité des données comptables)  
**Epic:** EP-018-S05  

---

## Overview

A *budget rectificatif* (amendment budget) is a distinct legal act voted in a General Assembly (AG) that corrects or amends an already approved budget. It does not modify the original budget in place — instead, it creates a new budget record linked to the original via `parent_budget_id`, and the budget lines are temporally versioned using `valid_from` / `valid_to`.

This document describes the complete workflow as implemented in the codebase, referencing ADR-009 Pattern B.

---

## State Model

```
┌─────────────────────────────────────────────────────────────────┐
│                        BUDGET STATES                               │
├─────────────────────────────────────────────────────────────────┤
│  draft    ──► approved ──► closed                                  │
│                            │                                       │
│                            │ create_rectificatif()                 │
│                            ▼                                       │
│                       draft (rectificatif)                        │
│                            │                                       │
│                            ▼                                       │
│                       approved (rectificatif)                     │
│                            │                                       │
│                            ▼                                       │
│                       closed (original)                           │
└─────────────────────────────────────────────────────────────────┘
```

- **draft**: Initial state, mutable
- **approved**: Budget has been voted and approved in AG, immutable for direct edits
- **closed**: Budget is no longer active (superseded by a rectificatif)

---

## Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant API
    participant DB
    participant AuditLog

    User->>API: POST /budgets/{original_id}/rectificatif
    Note right of User: Input: {approval_meeting_id}
    
    API->>DB: BEGIN transaction
    DB-->>API: tx
    
    API->>DB: SELECT budget WHERE id = original_id FOR UPDATE
    DB-->>API: original_budget (status='approved')
    
    alt original not approved
        API-->>User: 400 errors.budget.rectificatif_requires_approved
    end
    
    API->>DB: INSERT budget (parent_budget_id=original_id, status='draft')
    DB-->>API: rectificatif_budget
    
    API->>DB: UPDATE budget_line SET valid_to = yesterday
           WHERE budget_id = original_id AND valid_to IS NULL
    Note right of DB: Close all active lines from original
    
    API->>DB: INSERT budget_line (SELECT from original, valid_from=today, valid_to=NULL)
    Note right of DB: Copy lines to rectificatif as new active versions
    
    API->>AuditLog: INSERT (operation=CREATE, entity=budget, status=draft)
    
    API->>DB: COMMIT
    DB-->>API: success
    
    API-->>User: 201 Created (rectificatif in draft status)
    Note right of User: User can now edit rectificatif lines

    User->>API: PATCH /budgets/{rectificatif_id}/lines (edit lines as needed)
    API->>DB: UPDATE budget_line ...
    DB-->>API: success
    API-->>User: 200 OK

    User->>API: POST /budgets/{rectificatif_id}/approve
    Note right of User: Input: {approval_meeting_id}
    
    API->>DB: BEGIN transaction
    DB-->>API: tx
    
    API->>DB: UPDATE budget SET status='approved',
           approval_meeting_id = input.approval_meeting_id
           WHERE id = rectificatif_id AND status = 'draft'
    DB-->>API: approved_budget
    
    alt rectificatif has parent
        API->>DB: UPDATE budget SET status='closed'
               WHERE id = parent_id
        Note right of DB: Close original budget
        
        API->>DB: UPDATE budget_line SET valid_to = yesterday
               WHERE budget_id = parent_id AND valid_to IS NULL
        Note right of DB: Close any remaining open lines from parent
    end
    
    API->>AuditLog: INSERT (operation=UPDATE, entity=budget, status=approved)
    
    API->>DB: COMMIT
    DB-->>API: success
    
    API-->>User: 200 OK (rectificatif now approved, original now closed)
```

---

## Data Flow

### 1. Creating a Rectificatif (POST /budgets/{original_id}/rectificatif)

**Request:**
```json
{
  "approval_meeting_id": "uuid-of-the-ag-meeting"
}
```

**Database Operations:**
1. Verify original budget exists and has `status = 'approved'`
2. Create new budget row:
   - `building_id` = original.building_id
   - `fiscal_year` = original.fiscal_year
   - `status` = 'draft'
   - `parent_budget_id` = original.id
3. Close original budget's active lines:
   - `UPDATE budget_line SET valid_to = yesterday WHERE budget_id = original.id AND valid_to IS NULL`
4. Copy lines from original to rectificatif:
   - `INSERT INTO budget_line (budget_id, label, key_id, budgeted_amount, actual_amount, account_code, valid_from, valid_to)
     SELECT rectificatif.id, label, key_id, budgeted_amount, actual_amount, account_code, today, NULL
     FROM budget_line WHERE budget_id = original.id`
5. Insert audit log entry

**Response:** 201 Created with the new rectificatif budget in draft status

### 2. Approving a Rectificatif (POST /budgets/{rectificatif_id}/approve)

**Request:**
```json
{
  "approval_meeting_id": "uuid-of-the-ag-meeting"
}
```

**Database Operations:**
1. Update rectificatif budget:
   - `status` = 'approved'
   - `approval_meeting_id` = input.approval_meeting_id
2. If rectificatif has `parent_budget_id`:
   - Close parent budget: `UPDATE budget SET status = 'closed' WHERE id = parent_budget_id`
   - Close any remaining open lines from parent: `UPDATE budget_line SET valid_to = yesterday WHERE budget_id = parent_id AND valid_to IS NULL`
3. Insert audit log entry

**Response:** 200 OK with the approved rectificatif budget

---

## Temporal Versioning (valid_from / valid_to)

The `budget_line` table uses temporal versioning to track the history of budget line values:

- `valid_from`: The date from which this version is effective (inclusive)
- `valid_to`: The date until which this version is effective (exclusive), NULL means currently active

### Querying Active Lines

The `list_lines` endpoint filters to only return lines where `valid_to IS NULL`:

```sql
SELECT * FROM budget_line 
WHERE budget_id = $1 AND valid_to IS NULL
```

This ensures that only the currently active version of each line is exposed through the API.

### Version Chain Example

```
Original Budget (approved):
  budget_line id=1: valid_from=2026-01-01, valid_to=2026-08-04

Rectificatif created on 2026-08-05:
  budget_line id=1: valid_from=2026-01-01, valid_to=2026-08-04 (closed by rectificatif creation)
  budget_line id=2: valid_from=2026-08-05, valid_to=NULL (active, belongs to rectificatif)

Rectificatif approved:
  Original budget: status = 'closed'
  Rectificatif budget: status = 'approved'
```

---

## Constraints

### Database Constraints

1. **Partial unique index on root budgets:**
   ```sql
   CREATE UNIQUE INDEX budget_unique_root_per_year
       ON budget (building_id, fiscal_year)
       WHERE parent_budget_id IS NULL;
   ```
   Ensures only one root budget per building per fiscal year.

2. **Partial unique index on active budget lines:**
   ```sql
   CREATE UNIQUE INDEX budget_line_unique_active
       ON budget_line (budget_id, key_id, label)
       WHERE valid_to IS NULL;
   ```
   Ensures only one active version per (budget_id, key_id, label) combination.

### Business Rules

1. A rectificatif can only be created from an **approved** budget
2. A rectificatif is created in **draft** status (not auto-approved)
3. When a rectificatif is approved, its parent budget is automatically **closed**
4. Budget lines from the original are **closed** (valid_to set) when the rectificatif is created
5. The rectificatif gets fresh copies of all lines with new `valid_from` = creation date

---

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/budgets/{id}/rectificatif` | POST | Create a rectificatif from an approved budget |
| `/budgets/{id}/approve` | POST | Approve a draft budget (including rectificatifs) |
| `/budgets/{id}/lines` | GET | List active lines for a budget |
| `/budgets/{id}/lines` | POST | Create a new line in a draft budget |

---

## Error Codes

| Code | HTTP | Description |
|------|------|-------------|
| `errors.budget.rectificatif_requires_approved` | 400 | Original budget must be in 'approved' status |
| `errors.budget.not_found` | 404 | Budget not found |
| `errors.budget.not_mutable` | 400 | Budget is not in a mutable state |

---

## See Also

- [ADR-009 — Immutabilité des données comptables et audit log](ADR-009-accounting-immutability.md)
- [LEGAL.md §11 — Obligations légales de conservation](LEGAL.md#11-obligations-légales-de-conservation)
- [PROGRESS.md — Implementation tracking](PROGRESS.md)
