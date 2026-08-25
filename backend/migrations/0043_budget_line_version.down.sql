-- EP-018-S06: Revert version column for optimistic locking on budget_line

DROP TRIGGER IF EXISTS budget_line_version_trigger ON budget_line;
DROP FUNCTION IF EXISTS increment_budget_line_version();
ALTER TABLE budget_line DROP COLUMN IF EXISTS version;
