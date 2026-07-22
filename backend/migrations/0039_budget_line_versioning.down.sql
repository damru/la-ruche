-- EP-018-S06: revert budget_line temporal versioning (Pattern B)

DROP INDEX IF EXISTS budget_line_unique_active;
ALTER TABLE budget_line DROP COLUMN IF EXISTS valid_to;
ALTER TABLE budget_line DROP COLUMN IF EXISTS valid_from;
