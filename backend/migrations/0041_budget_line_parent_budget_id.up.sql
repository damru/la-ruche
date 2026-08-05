-- EP-018-S06: Pattern B — budget_line parent_budget_id
--
-- Links a budget_line to the parent budget in a rectificatif chain.
-- For root budget lines (no rectificatif), parent_budget_id IS NULL.
-- This allows fast parent-child lookups for budget line hierarchies.
--
-- No data migration needed; column is nullable for backward compatibility.

ALTER TABLE budget_line
    ADD COLUMN parent_budget_id UUID REFERENCES budget(id);

CREATE INDEX budget_line_parent_budget_id_id_idx
    ON budget_line (parent_budget_id, id);
