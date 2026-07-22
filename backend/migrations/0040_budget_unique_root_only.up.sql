-- EP-018-S06: Pattern B — budget uniqueness compatible with rectificatifs (part 3)
--
-- The original UNIQUE (building_id, fiscal_year) constraint forbade more
-- than one budget per building per fiscal year, which is incompatible with
-- rectificatifs: a rectificatif is a new budget row (parent_budget_id set
-- to the original) for the same building/fiscal_year. Replace the table
-- constraint with a partial unique index that only applies to root budgets
-- (parent_budget_id IS NULL), preserving the single-budget-per-year
-- invariant for roots while allowing any number of rectificatifs per root.
-- See ADR-009 §Pattern B.

ALTER TABLE budget
    DROP CONSTRAINT budget_building_id_fiscal_year_key;

CREATE UNIQUE INDEX budget_unique_root_per_year
    ON budget (building_id, fiscal_year)
    WHERE parent_budget_id IS NULL;
