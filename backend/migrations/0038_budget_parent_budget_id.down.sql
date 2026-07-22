-- EP-018-S06: revert budget parent_budget_id (Pattern B)

DROP INDEX IF EXISTS budget_parent_budget_id_idx;
ALTER TABLE budget DROP COLUMN IF EXISTS parent_budget_id;
