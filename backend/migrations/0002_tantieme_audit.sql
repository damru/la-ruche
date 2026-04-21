-- ============================================================
-- §2.2 — Versioning des tantièmes (LEGAL.md §2.2)
-- Adds valid_from / valid_to to unit_allocation_key so that
-- any change to a unit's share_value for a given key is
-- preserved as an immutable history record.
-- ============================================================

-- Rename existing table to preserve data during migration
ALTER TABLE unit_allocation_key RENAME TO unit_allocation_key_old;

CREATE TABLE unit_allocation_key (
    id          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    unit_id     UUID        NOT NULL REFERENCES unit(id),
    key_id      UUID        NOT NULL REFERENCES allocation_key(id),
    share_value INTEGER     NOT NULL CHECK (share_value >= 0),
    valid_from  DATE        NOT NULL DEFAULT CURRENT_DATE,
    valid_to    DATE,                   -- NULL = current active record
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Migrate existing data: treat all current rows as active (valid_to = NULL)
INSERT INTO unit_allocation_key (unit_id, key_id, share_value, valid_from)
SELECT unit_id, key_id, share_value, CURRENT_DATE
FROM unit_allocation_key_old;

DROP TABLE unit_allocation_key_old;

-- Ensures at most one active record per (unit, key) at any time
CREATE UNIQUE INDEX uak_unique_active
    ON unit_allocation_key (unit_id, key_id)
    WHERE valid_to IS NULL;

-- ============================================================
-- §2.3 — Snapshot des tantièmes dans les appels de fonds
-- (LEGAL.md §2.3)
-- Adds three columns to levy_call so each call permanently
-- records the allocation key used and the share counts at
-- generation time. Mandatory for audit and individual charge
-- statements (LEGAL.md §4.6).
-- ============================================================

ALTER TABLE levy_call
    ADD COLUMN key_id                UUID    REFERENCES allocation_key(id),
    ADD COLUMN shares_snapshot       INTEGER,   -- tantièmes du lot au moment de la génération
    ADD COLUMN shares_total_snapshot INTEGER;   -- total tantièmes de la clé au moment de la génération

COMMENT ON COLUMN levy_call.shares_snapshot       IS 'Snapshot of the unit share_value at generation time — immutable after INSERT';
COMMENT ON COLUMN levy_call.shares_total_snapshot IS 'Snapshot of the key total shares at generation time — immutable after INSERT';
