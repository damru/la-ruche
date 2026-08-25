-- EP-018-S06: Remove CHECK constraint for rectificatif_version >= 0
--
-- Step 1: Drop the CHECK constraint
ALTER TABLE budget_line
    DROP CONSTRAINT IF EXISTS budget_line_rectificatif_version_non_negative;
