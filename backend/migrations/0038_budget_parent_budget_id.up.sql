-- EP-018-S06: Pattern B — budget rectificatif (part 1)
--
-- A budget rectificatif is a distinct legal act voted in AG, not an
-- in-place edit of the approved budget. Adding parent_budget_id turns
-- `budget` into an explicit chain: a rectificatif budget references its
-- original (approved) budget. The root budget of a chain has
-- parent_budget_id NULL. See ADR-009 §Pattern B.
--
-- All pre-migration rows are chain roots (no rectificatif concept existed
-- before this migration) — no backfill needed, they keep parent_budget_id
-- NULL by default.

ALTER TABLE budget
    ADD COLUMN parent_budget_id UUID REFERENCES budget(id);

CREATE INDEX budget_parent_budget_id_idx
    ON budget (parent_budget_id);
