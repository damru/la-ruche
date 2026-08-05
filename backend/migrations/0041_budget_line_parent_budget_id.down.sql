-- EP-018-S06: Pattern B — budget_line parent_budget_id

DROP INDEX IF EXISTS budget_line_parent_budget_id_id_idx;

ALTER TABLE budget_line
    DROP COLUMN IF EXISTS parent_budget_id;
