-- Add draft value to claim_status enum. Value only — consumers in next migration.
ALTER TYPE claim_status ADD VALUE IF NOT EXISTS 'draft' BEFORE 'open';
