-- EP-002-S11: Governance register (LEGAL.md §1.7, §1.8)
--
-- Register of the syndic in office and council members with mandate periods,
-- to audit the separation of organs (art. 17 / 21 loi 1965).
-- Invariant: at most one active syndic mandate per building at any time.

CREATE TYPE governance_role AS ENUM (
    'syndic',
    'council_president',
    'council_member'
);

CREATE TABLE governance_mandate (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id   UUID NOT NULL REFERENCES building(id),
    role          governance_role NOT NULL,
    holder_name   TEXT NOT NULL,
    -- Optional link to an application account; mandate holders may be external.
    user_id       UUID REFERENCES user_account(id),
    start_date    DATE NOT NULL,
    end_date      DATE,                 -- NULL = open-ended (still in office)
    notes         TEXT,
    created_by    UUID NOT NULL REFERENCES user_account(id),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT governance_mandate_period_check
        CHECK (end_date IS NULL OR end_date >= start_date)
);

CREATE INDEX idx_governance_mandate_building
    ON governance_mandate(building_id, role);

-- DB backstop for "single active syndic": at most one open-ended syndic mandate
-- per building. Overlapping fixed-term syndic mandates are rejected in the domain layer.
CREATE UNIQUE INDEX uniq_active_syndic_per_building
    ON governance_mandate(building_id)
    WHERE role = 'syndic' AND end_date IS NULL;

CREATE TRIGGER trg_governance_mandate_updated_at
    BEFORE UPDATE ON governance_mandate
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
