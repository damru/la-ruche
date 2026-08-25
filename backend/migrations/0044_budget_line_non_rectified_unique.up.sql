-- EP-018-S06: Pattern B — budget_line non-rectified uniqueness constraint
--
-- Add partial unique index for non-rectified lines to prevent duplicates
-- within the same budget year. Non-rectified lines (rectificatif_version IS NULL)
-- must be unique on (budget_id, year, key_id, label) within a given year.
-- They can still duplicate across different years.
--
-- This complements the rectification-aware constraint from migration 0042
-- which only applies to rectified lines (rectificatif_version IS NOT NULL).

-- Verify no existing data would violate the new constraint
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM budget_line
        WHERE rectificatif_version IS NULL
        GROUP BY (budget_id, year, key_id, label)
        HAVING COUNT(*) > 1
    ) THEN
        RAISE EXCEPTION 'Duplicate non-rectified budget lines found for constraint on (budget_id, year, key_id, label) where rectificatif_version IS NULL';
    END IF;
END $$;

-- Create partial unique index for non-rectified lines
CREATE UNIQUE INDEX idx_budget_line_non_rectified_unique
    ON budget_line (budget_id, year, key_id, label)
    WHERE rectificatif_version IS NULL;
