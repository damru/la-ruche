-- EP-018-S06: Pattern B — budget_line temporal versioning (part 2)
--
-- A budget rectificatif corrects budgeted amounts without erasing history:
-- the affected budget_line rows are versioned with valid_from/valid_to
-- (same pattern as unit_allocation_key, ADR-006), rather than edited in
-- place. valid_to IS NULL marks the row currently in effect.
--
-- All pre-migration rows are backfilled as open-ended current versions:
-- valid_from = the row's creation date, valid_to = NULL.
--
-- Logical identity of a line across versions is (budget_id, key_id, label):
-- the same budget can have at most one currently-active row for a given
-- (key_id, label) pair. See ADR-009 §Versioning temporel.

ALTER TABLE budget_line
    ADD COLUMN valid_from DATE NOT NULL DEFAULT CURRENT_DATE,
    ADD COLUMN valid_to   DATE;

UPDATE budget_line SET valid_from = created_at::date;

CREATE UNIQUE INDEX budget_line_unique_active
    ON budget_line (budget_id, key_id, label)
    WHERE valid_to IS NULL;
