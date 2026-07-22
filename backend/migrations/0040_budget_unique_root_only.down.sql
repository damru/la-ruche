-- EP-018-S06: revert budget uniqueness change (Pattern B)

DROP INDEX IF EXISTS budget_unique_root_per_year;

ALTER TABLE budget
    ADD CONSTRAINT budget_building_id_fiscal_year_key UNIQUE (building_id, fiscal_year);
