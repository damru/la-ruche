-- EP-018-S06: Add CHECK constraint for rectificatif_version >= 0
--
-- Add CHECK constraint to ensure rectificatif_version >= 0.
-- The column already exists as rectificatif_version (added in migration 0042).
--
-- See ADR-009 §Pattern B.

-- Step 1: Add CHECK constraint
ALTER TABLE budget_line
    ADD CONSTRAINT budget_line_rectificatif_version_non_negative
    CHECK (rectificatif_version IS NULL OR rectificatif_version >= 0);
