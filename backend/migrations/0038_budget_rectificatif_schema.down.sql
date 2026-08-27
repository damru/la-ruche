-- EP-018-S06: revert budget rectificatif schema.

DROP TRIGGER IF EXISTS budget_line_version_trigger ON budget_line;
DROP FUNCTION IF EXISTS increment_budget_line_version();

ALTER TABLE budget_line
    DROP COLUMN IF EXISTS version,
    DROP COLUMN IF EXISTS rectificatif_version,
    DROP COLUMN IF EXISTS valid_to,
    DROP COLUMN IF EXISTS valid_from;

ALTER TABLE budget
    DROP COLUMN IF EXISTS parent_budget_id;
