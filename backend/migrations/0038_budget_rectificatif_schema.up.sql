-- EP-018-S06: Pattern B — budget rectificatif schema (ADR-009).
--
-- A budget rectificatif is a *new* budget row for the same building and fiscal
-- year, linked to the budget it amends via `parent_budget_id`. The amended
-- budget is never edited in place: its lines are closed with `valid_to` and
-- copied into the rectificatif, preserving the accounting history required by
-- LEGAL.md §11 (conservation 10 ans, non-rétroactivité).
--
-- Logical identity of a budget_line across versions is (budget_id, key_id,
-- label) — same pattern as unit_allocation_key (ADR-006). `valid_to IS NULL`
-- marks the version currently in effect.
--
-- `version` is optimistic-locking metadata only, unrelated to the temporal
-- versioning above: it guards concurrent edits of a draft line.

ALTER TABLE budget
    ADD COLUMN parent_budget_id UUID REFERENCES budget (id);

ALTER TABLE budget_line
    ADD COLUMN valid_from            DATE    NOT NULL DEFAULT CURRENT_DATE,
    ADD COLUMN valid_to              DATE,
    ADD COLUMN rectificatif_version  INTEGER,
    ADD COLUMN version               INTEGER NOT NULL DEFAULT 1;

-- Pre-existing rows are open-ended current versions dated from their creation.
UPDATE budget_line SET valid_from = created_at::date;

-- Optimistic locking. The trigger deliberately does NOT fire on the `valid_to`
-- closing performed by the rectificatif workflow: closing a line is a system
-- operation and must not invalidate a client's concurrency token.
CREATE OR REPLACE FUNCTION increment_budget_line_version()
    RETURNS TRIGGER AS $$
BEGIN
    NEW.version = OLD.version + 1;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER budget_line_version_trigger
    BEFORE UPDATE ON budget_line
    FOR EACH ROW
    WHEN (OLD.valid_to IS NOT DISTINCT FROM NEW.valid_to)
EXECUTE FUNCTION increment_budget_line_version();
