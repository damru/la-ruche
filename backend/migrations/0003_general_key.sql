-- ============================================================
-- ADR-007 — Clé de répartition générale système
-- Adds key_type enum ('general' | 'special') and soft delete
-- (deleted_at) to allocation_key.
-- ============================================================

-- Step 1: Create the enum type
CREATE TYPE allocation_key_type AS ENUM ('general', 'special');

-- Step 2: Add columns
ALTER TABLE allocation_key
    ADD COLUMN key_type    allocation_key_type NOT NULL DEFAULT 'special',
    ADD COLUMN deleted_at  TIMESTAMPTZ;

-- Step 3: Migrate existing 'Général' key → rename + mark as general
UPDATE allocation_key
SET name     = 'Charges Générales',
    key_type = 'general'
WHERE name = 'Général';

-- Step 4: Enforce at most one active general key per building
CREATE UNIQUE INDEX allocation_key_one_general
    ON allocation_key (building_id)
    WHERE key_type = 'general' AND deleted_at IS NULL;
