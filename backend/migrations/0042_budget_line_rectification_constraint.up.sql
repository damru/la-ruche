-- EP-018-S06: Pattern B — budget_line rectification-aware constraint
--
-- Replace the existing unique constraint on budget_line to support rectificatifs.
-- 
-- The existing constraint (budget_id, key_id, label) WHERE valid_to IS NULL
-- prevents duplicate active lines, but doesn't account for rectification numbering.
-- 
-- New design:
-- - Add year column (denormalized from budget.fiscal_year for easier querying)
-- - Add line_type column (to categorize lines: 'charge', 'produit', 'fonds_travaux', etc.)
-- - Add rectification_number column (NULL for root budget lines, 1..N for rectificatifs)
-- - Drop the old unique constraint
-- - Create partial unique index that allows:
--   * Multiple rectificatifs per year (each with unique rectification_number)
--   * Non-rectified lines to be free to duplicate across years
--   * Only rectified lines (rectification_number IS NOT NULL) are constrained
--
-- See ADR-009 §Pattern B.

-- Step 1: Add new columns
ALTER TABLE budget_line
    ADD COLUMN year INTEGER,
    ADD COLUMN line_type TEXT,
    ADD COLUMN rectification_number INTEGER;

-- Step 2: Backfill year from budget.fiscal_year
UPDATE budget_line bl
    SET year = b.fiscal_year
    FROM budget b
    WHERE bl.budget_id = b.id;

-- Step 3: Backfill line_type (default to 'charge' for existing lines)
-- This is a safe default; actual line types should be set by the application
UPDATE budget_line
    SET line_type = 'charge'
    WHERE line_type IS NULL;

-- Step 4: For root budget lines (not part of a rectificatif), set rectification_number to NULL
-- For lines that are part of a rectificatif chain, we need to determine the rectification number
-- from the parent_budget_id chain. This is a simplified backfill.
-- 
-- Root budget lines: those where parent_budget_id IS NULL (or not set, since it's a recent addition)
-- For now, set all existing lines as non-rectified (rectification_number = NULL)
-- The application will set rectification_number when creating rectificatifs
UPDATE budget_line
    SET rectification_number = NULL
    WHERE rectification_number IS NULL;

-- Step 5: Verify no existing data would violate the new constraint
-- After backfill, all existing lines have rectification_number = NULL
-- The new constraint only applies when rectification_number IS NOT NULL
-- So we check if there are any lines with rectification_number IS NOT NULL that would violate uniqueness
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM budget_line
        WHERE rectification_number IS NOT NULL
        GROUP BY (budget_id, year, line_type, rectification_number)
        HAVING COUNT(*) > 1
    ) THEN
        RAISE EXCEPTION 'Duplicate budget lines found for rectification constraint on (budget_id, year, line_type, rectification_number) where rectification_number IS NOT NULL';
    END IF;
END $$;

-- Step 6: Drop the existing unique constraint
DROP INDEX IF EXISTS budget_line_unique_active;

-- Step 7: Create the new partial unique index for rectification-aware constraint
-- This ensures that within a budget year, for rectified lines (rectification_number IS NOT NULL),
-- the combination of (budget_id, year, line_type, rectification_number) is unique.
-- Non-rectified lines (rectification_number IS NULL) are not constrained by this index.
CREATE UNIQUE INDEX idx_budget_line_rectif
    ON budget_line (budget_id, year, line_type, rectification_number)
    WHERE rectification_number IS NOT NULL;
