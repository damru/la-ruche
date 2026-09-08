-- EP-018-S06: revert budget rectificatif invariants.
--
-- Restoring UNIQUE (building_id, fiscal_year) is only possible while no
-- rectificatif exists. Abort loudly rather than leave the schema half-reverted.

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM budget WHERE parent_budget_id IS NOT NULL) THEN
        RAISE EXCEPTION 'Cannot revert 0039: rectificatif budgets exist. Delete them before reverting, or the UNIQUE (building_id, fiscal_year) constraint cannot be restored.';
    END IF;
END $$;

ALTER TABLE budget_line
    DROP CONSTRAINT IF EXISTS budget_line_rectificatif_version_positive,
    DROP CONSTRAINT IF EXISTS budget_line_valid_period;

DROP INDEX IF EXISTS budget_line_unique_active;
DROP INDEX IF EXISTS budget_parent_budget_id_idx;
DROP INDEX IF EXISTS budget_unique_approved_per_year;
DROP INDEX IF EXISTS budget_unique_pending_rectificatif;
DROP INDEX IF EXISTS budget_unique_root_per_year;

ALTER TABLE budget
    ADD CONSTRAINT budget_building_id_fiscal_year_key UNIQUE (building_id, fiscal_year);
