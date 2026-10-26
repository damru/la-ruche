-- EP-008-S06: claim draft state (PRD US-SIN-001b). Value only — consumers in next migration.
ALTER TYPE claim_status ADD VALUE IF NOT EXISTS 'draft' BEFORE 'open';
