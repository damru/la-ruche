-- 0007_contractor_referential.sql
-- Adds phone to user_account, creates the contractor referential table,
-- and links tasks to a contractor from the referential.

-- ── User phone ────────────────────────────────────────────────────────────────
ALTER TABLE user_account ADD COLUMN IF NOT EXISTS phone TEXT;

-- ── Contractor referential ────────────────────────────────────────────────────
-- Scoped per building. Accessible to all users (read), manageable by managers.
CREATE TABLE contractor (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id UUID NOT NULL REFERENCES building(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    email       TEXT,
    phone       TEXT,
    domain      TEXT,           -- professional domain (free text, e.g. 'plomberie')
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contractor_building ON contractor(building_id);
-- Partial unique index: prevent duplicate contractor names per building
CREATE UNIQUE INDEX idx_contractor_building_name ON contractor(building_id, lower(name));

-- ── Link task to contractor ───────────────────────────────────────────────────
ALTER TABLE task ADD COLUMN contractor_id UUID REFERENCES contractor(id) ON DELETE SET NULL;
