-- EP-002-S10: Judicial proceedings register (LEGAL.md §7.1, §10)
--
-- Tracks proceedings involving the syndicat (as claimant or defendant).
-- Feeds the état daté (EP-002-S06): ongoing proceedings of a building.

CREATE TYPE judicial_proceeding_type AS ENUM (
    'debt_recovery',   -- recouvrement de charges (art. 19-2)
    'works_dispute',   -- litige travaux / malfaçon
    'liability',       -- responsabilité / dommages
    'other'
);

CREATE TYPE judicial_proceeding_status AS ENUM (
    'ongoing',
    'suspended',
    'closed'
);

CREATE TABLE judicial_proceeding (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id     UUID NOT NULL REFERENCES building(id),
    proceeding_type judicial_proceeding_type NOT NULL DEFAULT 'other',
    status          judicial_proceeding_status NOT NULL DEFAULT 'ongoing',
    subject         TEXT NOT NULL,
    claimant        TEXT NOT NULL,
    defendant       TEXT NOT NULL,
    jurisdiction    TEXT,
    case_reference  TEXT,
    opened_at       DATE NOT NULL,
    closed_at       DATE,
    outcome         TEXT,
    created_by      UUID NOT NULL REFERENCES user_account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT judicial_proceeding_closed_after_opened_check
        CHECK (closed_at IS NULL OR closed_at >= opened_at),
    CONSTRAINT judicial_proceeding_closed_consistency_check
        CHECK (
            (status = 'closed' AND closed_at IS NOT NULL)
            OR (status <> 'closed' AND closed_at IS NULL)
        )
);

CREATE INDEX idx_judicial_proceeding_building
    ON judicial_proceeding(building_id, status);

CREATE TRIGGER trg_judicial_proceeding_updated_at
    BEFORE UPDATE ON judicial_proceeding
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
