-- ============================================================
-- La Ruche — PostgreSQL Schema
-- Licence : AGPL-3.0
-- ============================================================
-- Conventions:
--   - UUID v4 for all ids
--   - snake_case everywhere
--   - Timestamps always TIMESTAMPTZ
--   - Soft delete via "active" or "status" column depending on entity
--   - JSONB for semi-structured data (OCR extraction)
--   - PG arrays for roles and file paths
-- ============================================================

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================
-- ENUM TYPES
-- ============================================================

CREATE TYPE unit_type AS ENUM (
    'apartment', 'cellar', 'parking', 'commercial_unit', 'other'
);

CREATE TYPE user_role AS ENUM (
    'manager', 'board_member', 'co_owner', 'tenant'
);

CREATE TYPE meeting_status AS ENUM (
    'draft', 'called', 'held', 'closed'
);

CREATE TYPE meeting_type AS ENUM (
    'annual', 'extraordinary'
);

CREATE TYPE vote_type AS ENUM (
    'simple_majority',   -- Art. 24: majority of votes present/represented
    'absolute_majority', -- Art. 25: absolute majority of all co-owners
    'double_majority',   -- Art. 26: double majority
    'unanimity'
);

CREATE TYPE resolution_status AS ENUM (
    'pending', 'passed', 'rejected', 'withdrawn'
);

CREATE TYPE vote_position AS ENUM (
    'in_favor', 'against', 'abstain', 'non_voter'
);

CREATE TYPE budget_status AS ENUM (
    'draft', 'approved', 'closed'
);

CREATE TYPE payment_frequency AS ENUM (
    'monthly', 'quarterly', 'biannual', 'annual'
);

CREATE TYPE levy_status AS ENUM (
    'issued', 'paid', 'overdue'
);

CREATE TYPE task_status AS ENUM (
    'open', 'in_progress', 'awaiting_vendor', 'closed', 'cancelled'
);

CREATE TYPE task_priority AS ENUM (
    'low', 'normal', 'high', 'urgent'
);

CREATE TYPE claim_status AS ENUM (
    'open', 'in_progress', 'closed'
);

CREATE TYPE report_status AS ENUM (
    'open', 'in_progress', 'closed'
);

CREATE TYPE poll_status AS ENUM (
    'draft', 'open', 'closed'
);

CREATE TYPE contract_type AS ENUM (
    'maintenance', 'insurance', 'service', 'other'
);

CREATE TYPE contract_status AS ENUM (
    'active', 'terminated', 'expired'
);

CREATE TYPE reminder_type AS ENUM (
    'renewal', 'payment_due', 'expiration', 'termination'
);

CREATE TYPE reminder_status AS ENUM (
    'pending', 'sent', 'acknowledged'
);

CREATE TYPE log_source AS ENUM (
    'manual', 'automatic'
);

CREATE TYPE extraction_status AS ENUM (
    'pending', 'complete', 'failed'
);

-- ============================================================
-- BUILDING (copropriete)
-- ============================================================

CREATE TABLE building (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            TEXT NOT NULL,
    address_line1   TEXT NOT NULL,
    address_line2   TEXT,
    postal_code     TEXT NOT NULL,
    city            TEXT NOT NULL,
    manager_siret   TEXT,                        -- optional for volunteer managers
    shares_total    NUMERIC(10, 2) NOT NULL,      -- base 1000 or 10000
    bylaws_date     DATE,                         -- date of condominium bylaws
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- UNIT (lot)
-- ============================================================

CREATE TABLE unit (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id     UUID NOT NULL REFERENCES building(id),
    unit_number     TEXT NOT NULL,               -- official number from bylaws
    unit_type       unit_type NOT NULL,
    block           TEXT,
    staircase       TEXT,
    floor           TEXT,
    description     TEXT,
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (building_id, unit_number)
);

-- ============================================================
-- ALLOCATION KEYS (cle_repartition)
-- ============================================================

CREATE TABLE allocation_key (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id     UUID NOT NULL REFERENCES building(id),
    name            TEXT NOT NULL,               -- e.g. "General", "Elevator", "Cold Water"
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (building_id, name)
);

CREATE TABLE unit_allocation_key (
    unit_id         UUID NOT NULL REFERENCES unit(id),
    key_id          UUID NOT NULL REFERENCES allocation_key(id),
    share_value     NUMERIC(10, 4) NOT NULL,     -- shares of this unit for this key
    PRIMARY KEY (unit_id, key_id)
);

-- ============================================================
-- USER ACCOUNTS (utilisateur)
-- ============================================================

CREATE TABLE user_account (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email               TEXT NOT NULL UNIQUE,
    last_name           TEXT NOT NULL,
    first_name          TEXT NOT NULL,
    password_hash       TEXT,                    -- nullable if passkey only
    passkey_credential  JSONB,                   -- webauthn credential store
    active              BOOLEAN NOT NULL DEFAULT TRUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Invitation / reset password tokens
CREATE TABLE user_token (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID REFERENCES user_account(id),  -- nullable before account creation
    email           TEXT NOT NULL,
    token_type      TEXT NOT NULL,               -- 'invitation' | 'reset_password' | 'refresh'
    token_hash      TEXT NOT NULL UNIQUE,        -- hash of token sent by email
    building_id     UUID REFERENCES building(id),
    invited_role    user_role,                   -- role to assign on invitation
    expires_at      TIMESTAMPTZ NOT NULL,
    used_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Roles per building (cumulative)
CREATE TABLE user_building (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES user_account(id),
    building_id     UUID NOT NULL REFERENCES building(id),
    roles           user_role[] NOT NULL,        -- e.g. {co_owner, board_member}
    start_date      DATE NOT NULL DEFAULT CURRENT_DATE,
    end_date        DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, building_id)
);

-- ============================================================
-- OWNERSHIP HISTORY (lot_proprietaire) — GDPR
-- ============================================================

CREATE TABLE unit_owner (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    unit_id         UUID NOT NULL REFERENCES unit(id),
    user_id         UUID REFERENCES user_account(id),  -- NULL after anonymisation
    anonymized_name TEXT,                        -- "Anonymised owner" after GDPR erasure
    start_date      DATE NOT NULL,
    end_date        DATE,                        -- NULL = current owner
    anonymized_at   TIMESTAMPTZ,                 -- date of GDPR request
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- DOCUMENTS
-- ============================================================

CREATE TABLE document (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id         UUID NOT NULL REFERENCES building(id),
    original_name       TEXT NOT NULL,           -- original filename (stored only here)
    storage_path        TEXT NOT NULL,           -- UUID + extension in S3 bucket
    bucket              TEXT NOT NULL,           -- per-building isolation
    mime_type           TEXT NOT NULL,
    size_bytes          BIGINT NOT NULL,
    visible_to_roles    user_role[] NOT NULL,    -- access control
    uploaded_by         UUID REFERENCES user_account(id),
    -- OCR/NLP (post-MVP)
    extraction_status   extraction_status,
    extraction_data     JSONB,
    extraction_at       TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- GENERAL MEETING (assemblee_generale)
-- ============================================================

CREATE TABLE general_meeting (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id         UUID NOT NULL REFERENCES building(id),
    meeting_type        meeting_type NOT NULL,
    status              meeting_status NOT NULL DEFAULT 'draft',
    notice_date         DATE,
    meeting_date        TIMESTAMPTZ NOT NULL,
    location            TEXT,
    agenda              TEXT,
    minutes_document_id UUID REFERENCES document(id),
    created_by          UUID NOT NULL REFERENCES user_account(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE resolution (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    meeting_id  UUID NOT NULL REFERENCES general_meeting(id),
    number      INTEGER NOT NULL,
    title       TEXT NOT NULL,
    description TEXT,
    vote_type   vote_type NOT NULL,
    status      resolution_status NOT NULL DEFAULT 'pending',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (meeting_id, number)
);

CREATE TABLE vote (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resolution_id       UUID NOT NULL REFERENCES resolution(id),
    user_id             UUID NOT NULL REFERENCES user_account(id),
    unit_id             UUID NOT NULL REFERENCES unit(id),
    position            vote_position NOT NULL,
    shares_represented  NUMERIC(10, 4) NOT NULL,
    proxy_for           UUID REFERENCES user_account(id),  -- NULL = direct vote
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (resolution_id, unit_id)              -- one vote per unit per resolution
);

-- ============================================================
-- BUDGET & LEVY CALLS
-- ============================================================

CREATE TABLE budget (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id         UUID NOT NULL REFERENCES building(id),
    fiscal_year         INTEGER NOT NULL,
    status              budget_status NOT NULL DEFAULT 'draft',
    approval_meeting_id UUID REFERENCES general_meeting(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (building_id, fiscal_year)
);

CREATE TABLE budget_line (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    budget_id       UUID NOT NULL REFERENCES budget(id),
    label           TEXT NOT NULL,
    key_id          UUID NOT NULL REFERENCES allocation_key(id),
    budgeted_amount NUMERIC(12, 2) NOT NULL,
    actual_amount   NUMERIC(12, 2),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE levy_call (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    budget_id   UUID NOT NULL REFERENCES budget(id),
    unit_id     UUID NOT NULL REFERENCES unit(id),
    amount      NUMERIC(12, 2) NOT NULL,
    due_date    DATE NOT NULL,
    status      levy_status NOT NULL DEFAULT 'issued',
    issued_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paid_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Works fund — Art. 14-2 ALUR (mandatory separate bank account)
CREATE TABLE works_fund (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id         UUID NOT NULL REFERENCES building(id),
    fiscal_year         INTEGER NOT NULL,
    levied_amount       NUMERIC(12, 2) NOT NULL DEFAULT 0,
    available_amount    NUMERIC(12, 2) NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (building_id, fiscal_year)
);

-- ============================================================
-- VENDOR CONTRACTS (contrat_fournisseur)
-- ============================================================

CREATE TABLE vendor_contract (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id         UUID NOT NULL REFERENCES building(id),
    vendor_name         TEXT NOT NULL,
    vendor_email        TEXT,
    vendor_phone        TEXT,
    subject             TEXT NOT NULL,
    contract_type       contract_type NOT NULL,
    start_date          DATE NOT NULL,
    end_date            DATE,
    amount              NUMERIC(12, 2),
    payment_frequency   payment_frequency,
    auto_renewal        BOOLEAN NOT NULL DEFAULT FALSE,
    notice_days         INTEGER,                -- termination notice in days
    status              contract_status NOT NULL DEFAULT 'active',
    created_by          UUID NOT NULL REFERENCES user_account(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE contract_document (
    contract_id     UUID NOT NULL REFERENCES vendor_contract(id),
    document_id     UUID NOT NULL REFERENCES document(id),
    PRIMARY KEY (contract_id, document_id)
);

-- Reminders computed at contract creation/update
-- Triggered by poll: status='pending' AND reminder_date <= now()
CREATE TABLE contract_reminder (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contract_id     UUID NOT NULL REFERENCES vendor_contract(id),
    reminder_type   reminder_type NOT NULL,
    reminder_date   DATE NOT NULL,
    status          reminder_status NOT NULL DEFAULT 'pending',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- TASKS & VENDORS (tache / prestataire)
-- ============================================================

CREATE TABLE task (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id     UUID NOT NULL REFERENCES building(id),
    title           TEXT NOT NULL,
    description     TEXT,
    status          task_status NOT NULL DEFAULT 'open',
    priority        task_priority NOT NULL DEFAULT 'normal',
    created_by      UUID NOT NULL REFERENCES user_account(id),
    assigned_to     UUID REFERENCES user_account(id),
    contract_id     UUID REFERENCES vendor_contract(id),
    vendor_token    UUID UNIQUE,                -- ephemeral UUID generated on demand
    token_expires_at TIMESTAMPTZ,
    closed_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE vendor_submission (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_id     UUID NOT NULL REFERENCES task(id),
    comment     TEXT,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE vendor_submission_document (
    submission_id   UUID NOT NULL REFERENCES vendor_submission(id),
    document_id     UUID NOT NULL REFERENCES document(id),
    PRIMARY KEY (submission_id, document_id)
);

-- ============================================================
-- CLAIMS (sinistre)
-- ============================================================

CREATE TABLE claim (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id     UUID NOT NULL REFERENCES building(id),
    title           TEXT NOT NULL,
    description     TEXT,
    declared_at     DATE NOT NULL,
    status          claim_status NOT NULL DEFAULT 'open',
    unit_id         UUID REFERENCES unit(id),   -- nullable for common areas
    created_by      UUID NOT NULL REFERENCES user_account(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE claim_document (
    claim_id    UUID NOT NULL REFERENCES claim(id),
    document_id UUID NOT NULL REFERENCES document(id),
    PRIMARY KEY (claim_id, document_id)
);

-- ============================================================
-- MAINTENANCE LOG (carnet_entretien)
-- ============================================================

CREATE TABLE maintenance_log (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id         UUID NOT NULL REFERENCES building(id),
    title               TEXT NOT NULL,
    description         TEXT,
    intervention_date   DATE NOT NULL,
    cost                NUMERIC(12, 2),
    vendor_name         TEXT,                   -- free text
    contract_id         UUID REFERENCES vendor_contract(id),
    task_id             UUID REFERENCES task(id),
    source              log_source NOT NULL DEFAULT 'manual',
    created_by          UUID NOT NULL REFERENCES user_account(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE maintenance_log_document (
    log_id      UUID NOT NULL REFERENCES maintenance_log(id),
    document_id UUID NOT NULL REFERENCES document(id),
    PRIMARY KEY (log_id, document_id)
);

-- ============================================================
-- REPORTS (signalement)
-- ============================================================

CREATE TABLE report (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id UUID NOT NULL REFERENCES building(id),
    unit_id     UUID REFERENCES unit(id),
    author_id   UUID NOT NULL REFERENCES user_account(id),
    title       TEXT NOT NULL,
    description TEXT,
    status      report_status NOT NULL DEFAULT 'open',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE report_document (
    report_id   UUID NOT NULL REFERENCES report(id),
    document_id UUID NOT NULL REFERENCES document(id),
    PRIMARY KEY (report_id, document_id)
);

-- ============================================================
-- CONSULTATIONS / POLLS
-- ============================================================

CREATE TABLE consultation (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id     UUID NOT NULL REFERENCES building(id),
    title           TEXT NOT NULL,
    description     TEXT,
    status          poll_status NOT NULL DEFAULT 'draft',
    visible_to_roles user_role[] NOT NULL,      -- who can respond
    created_by      UUID NOT NULL REFERENCES user_account(id),
    closed_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE consultation_choice (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID NOT NULL REFERENCES consultation(id),
    label           TEXT NOT NULL,
    order_index     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE consultation_response (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID NOT NULL REFERENCES consultation(id),
    choice_id       UUID NOT NULL REFERENCES consultation_choice(id),
    user_id         UUID NOT NULL REFERENCES user_account(id),
    comment         TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (consultation_id, user_id)           -- one response per person
);

-- ============================================================
-- INDEXES
-- ============================================================

-- Access by building (most frequent queries)
CREATE INDEX idx_unit_building           ON unit(building_id);
CREATE INDEX idx_user_building_building  ON user_building(building_id);
CREATE INDEX idx_user_building_user      ON user_building(user_id);
CREATE INDEX idx_unit_owner_unit         ON unit_owner(unit_id);
CREATE INDEX idx_unit_owner_current      ON unit_owner(unit_id) WHERE end_date IS NULL;
CREATE INDEX idx_document_building       ON document(building_id);
CREATE INDEX idx_meeting_building        ON general_meeting(building_id);
CREATE INDEX idx_budget_building         ON budget(building_id);
CREATE INDEX idx_levy_call_unit          ON levy_call(unit_id);
CREATE INDEX idx_levy_call_status        ON levy_call(status) WHERE status != 'paid';
CREATE INDEX idx_task_building           ON task(building_id);
CREATE INDEX idx_task_token              ON task(vendor_token) WHERE vendor_token IS NOT NULL;
CREATE INDEX idx_reminder_trigger        ON contract_reminder(reminder_date, status)
    WHERE status = 'pending';                   -- partial index for poll job
CREATE INDEX idx_contract_building       ON vendor_contract(building_id);
CREATE INDEX idx_report_building         ON report(building_id);
CREATE INDEX idx_consultation_building   ON consultation(building_id);
CREATE INDEX idx_log_building            ON maintenance_log(building_id);
CREATE INDEX idx_user_token_hash         ON user_token(token_hash);
CREATE INDEX idx_user_token_active       ON user_token(expires_at) WHERE used_at IS NULL;

-- ============================================================
-- AUTOMATIC updated_at TRIGGERS
-- ============================================================

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_building_updated_at
    BEFORE UPDATE ON building
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_unit_updated_at
    BEFORE UPDATE ON unit
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_user_account_updated_at
    BEFORE UPDATE ON user_account
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_user_building_updated_at
    BEFORE UPDATE ON user_building
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_meeting_updated_at
    BEFORE UPDATE ON general_meeting
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_budget_updated_at
    BEFORE UPDATE ON budget
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_vendor_contract_updated_at
    BEFORE UPDATE ON vendor_contract
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_reminder_updated_at
    BEFORE UPDATE ON contract_reminder
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_task_updated_at
    BEFORE UPDATE ON task
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_claim_updated_at
    BEFORE UPDATE ON claim
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_log_updated_at
    BEFORE UPDATE ON maintenance_log
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_report_updated_at
    BEFORE UPDATE ON report
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER tultation_updated_at
    BEFORE UPDATE ON consultation
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_works_fund_updated_at
    BEFORE UPDATE ON works_fund
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
