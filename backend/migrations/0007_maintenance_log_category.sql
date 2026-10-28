-- EP-009-S10: logbook category (PRD US-CE-005).
-- `category` is a persisted business field (editable by the manager);
-- filtering stays client-side (see docs/specs/EP-009-S10-spec.md §2).
CREATE TYPE maintenance_category AS ENUM ('travaux', 'contrat', 'sinistre', 'autre');

ALTER TABLE maintenance_log
    ADD COLUMN category maintenance_category NOT NULL DEFAULT 'autre';

-- Backfill heuristic: linked contract => 'contrat', linked task => 'travaux', else 'autre'.
-- When a row links both, 'contrat' wins (UPDATE order is deterministic).
-- Note: 'sinistre' is never assigned by the backfill — maintenance_log has no
-- claim_id link column, so claim-related entries can only be categorised
-- manually at edit time.
UPDATE maintenance_log SET category = 'contrat' WHERE contract_id IS NOT NULL;
UPDATE maintenance_log SET category = 'travaux' WHERE task_id IS NOT NULL AND contract_id IS NULL;
