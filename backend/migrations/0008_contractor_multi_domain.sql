-- 0008_contractor_multi_domain.sql
-- Adds notes field and changes domain (single TEXT) to domains (TEXT array)
-- so a contractor can cover multiple domains.

-- ── Add notes field ───────────────────────────────────────────────────────────
ALTER TABLE contractor ADD COLUMN IF NOT EXISTS notes TEXT;

-- ── Add multi-domain array column ─────────────────────────────────────────────
ALTER TABLE contractor ADD COLUMN IF NOT EXISTS domains TEXT[] NOT NULL DEFAULT '{}';

-- ── Migrate existing single-domain value into the array ───────────────────────
UPDATE contractor SET domains = ARRAY[domain] WHERE domain IS NOT NULL AND domain <> '';

-- ── Drop the old single-domain column ────────────────────────────────────────
ALTER TABLE contractor DROP COLUMN IF EXISTS domain;
