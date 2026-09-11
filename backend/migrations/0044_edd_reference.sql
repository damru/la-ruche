-- EP-001-S09: manual EDD reference totals for coherence check (LEGAL.md §1.2)
-- Manual entry of EDD (état descriptif de division) reference tantièmes,
-- used by a read-only, non-blocking coherence check against configured
-- allocation key shares. document_id is nullable until EP-005-S06 typing
-- is wired up from the frontend (schema support already exists).

CREATE TABLE edd_reference (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id  UUID NOT NULL REFERENCES building(id),
    label        TEXT NOT NULL,
    edd_date     DATE NOT NULL,
    document_id  UUID NULL REFERENCES document(id),
    created_by   UUID NOT NULL REFERENCES user_account(id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE edd_reference_share (
    id                 UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    edd_reference_id   UUID NOT NULL REFERENCES edd_reference(id) ON DELETE CASCADE,
    unit_id            UUID NOT NULL REFERENCES unit(id),
    allocation_key_id  UUID NOT NULL REFERENCES allocation_key(id),
    share_value        NUMERIC(12,2) NOT NULL,
    UNIQUE (edd_reference_id, unit_id, allocation_key_id)
);

CREATE INDEX edd_reference_building_idx ON edd_reference (building_id, edd_date DESC, created_at DESC);
