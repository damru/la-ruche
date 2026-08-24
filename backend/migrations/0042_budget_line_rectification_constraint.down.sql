-- EP-018-S06: Pattern B — budget_line rectification-aware constraint (down)
--
-- Revert the rectification-aware constraint back to the temporal versioning constraint.

-- Step 1: Drop the new partial unique index
DROP INDEX IF EXISTS idx_budget_line_rectif;

-- Step 2: Restore the old unique constraint
CREATE UNIQUE INDEX budget_line_unique_active
    ON budget_line (budget_id, key_id, label)
    WHERE valid_to IS NULL;

-- Step 3: Drop the new columns
ALTER TABLE budget_line
    DROP COLUMN IF EXISTS year,
    DROP COLUMN IF EXISTS line_type,
    DROP COLUMN IF EXISTS rectification_number;
