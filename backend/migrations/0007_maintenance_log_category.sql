-- EP-009-S10: logbook category (PRD US-CE-005).
-- `category` is a persisted business field (editable by the manager);
-- filtering stays client-side (see docs/specs/EP-009-S10-spec.md §2).
CREATE TYPE maintenance_category AS ENUM ('works', 'contract', 'claim', 'other');

ALTER TABLE maintenance_log
    ADD COLUMN category maintenance_category NOT NULL DEFAULT 'other';

-- Backfill heuristic covering all categories:
--   1. linked contract        => 'contract'
--   2. linked task (no contract) => 'works'
--   3. keyword match on title/description => 'claim' (e.g. "sinistre", "dégât des eaux")
--   4. fallback               => 'other' (column default)
-- When a row links both contract and task, 'contract' wins (UPDATE order is deterministic).
UPDATE maintenance_log SET category = 'contract' WHERE contract_id IS NOT NULL;
UPDATE maintenance_log SET category = 'works' WHERE task_id IS NOT NULL AND contract_id IS NULL;
UPDATE maintenance_log SET category = 'claim'
WHERE category = 'other'
  AND (
    LOWER(title) LIKE '%sinistre%'
    OR LOWER(title) LIKE '%dégât%'
    OR LOWER(title) LIKE '%degat%'
    OR LOWER(title) LIKE '%inondation%'
    OR LOWER(title) LIKE '%fuite%'
    OR LOWER(COALESCE(description, '')) LIKE '%sinistre%'
    OR LOWER(COALESCE(description, '')) LIKE '%dégât%'
    OR LOWER(COALESCE(description, '')) LIKE '%degat%'
    OR LOWER(COALESCE(description, '')) LIKE '%inondation%'
  );
