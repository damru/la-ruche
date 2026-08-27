-- EP-018-S06: Pattern B — budget rectificatif invariants (ADR-009).
--
-- These indexes are the *only* guard on the invariants below: application-side
-- pre-checks would be check-then-act races. Constraint names are mapped to i18n
-- error codes in `db/budget.rs`.

-- The original UNIQUE (building_id, fiscal_year) forbade more than one budget
-- per year, which is incompatible with rectificatifs. The one-budget-per-year
-- rule is preserved for root budgets only.
ALTER TABLE budget
    DROP CONSTRAINT budget_building_id_fiscal_year_key;

CREATE UNIQUE INDEX budget_unique_root_per_year
    ON budget (building_id, fiscal_year)
    WHERE parent_budget_id IS NULL;

-- AC-BC-009-C: at most one rectificatif in preparation per amended budget.
CREATE UNIQUE INDEX budget_unique_pending_rectificatif
    ON budget (parent_budget_id)
    WHERE parent_budget_id IS NOT NULL AND status = 'draft';

-- AC-BC-009-C: at most one budget in force per fiscal year. `approve` closes
-- the amended budget before promoting the rectificatif, so the two never
-- overlap.
CREATE UNIQUE INDEX budget_unique_approved_per_year
    ON budget (building_id, fiscal_year)
    WHERE status = 'approved';

-- Walking a rectificatif chain (GET /budgets/{id}/rectificatifs).
CREATE INDEX budget_parent_budget_id_idx
    ON budget (parent_budget_id)
    WHERE parent_budget_id IS NOT NULL;

-- At most one version of a logical line in effect at any time. This is the
-- invariant the whole valid_from/valid_to design rests on.
CREATE UNIQUE INDEX budget_line_unique_active
    ON budget_line (budget_id, key_id, label)
    WHERE valid_to IS NULL;

ALTER TABLE budget_line
    ADD CONSTRAINT budget_line_valid_period
        CHECK (valid_to IS NULL OR valid_to >= valid_from),
    ADD CONSTRAINT budget_line_rectificatif_version_positive
        CHECK (rectificatif_version IS NULL OR rectificatif_version >= 1);
