-- ============================================================
-- La Ruche — PostgreSQL Schema (consolidated)
-- Generated from migrations 0001 → 0025
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

-- Lifecycle: draft → called → ongoing → closed  (0019 added 'ongoing'; 0020 removed 'held')
CREATE TYPE meeting_status AS ENUM (
    'draft', 'called', 'ongoing', 'closed'
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

-- 'awaiting_validation' added by migration 0012
CREATE TYPE task_status AS ENUM (
    'open', 'in_progress', 'awaiting_vendor', 'awaiting_validation', 'closed', 'cancelled'
);

CREATE TYPE task_priority AS ENUM (
    'low', 'normal', 'high', 'urgent'
);

CREATE TYPE claim_status AS ENUM (
    'open', 'in_progress', 'closed'
);

-- claim_type added by migration 0014
CREATE TYPE claim_type AS ENUM (
    'current',
    'natural_disaster',
    'theft'
);

CREATE TYPE report_status AS ENUM (
    'open', 'in_progress', 'closed'
);

CREATE TYPE poll_status AS ENUM (
    'draft', 'open', 'closed'
);

-- 'energy', 'water', 'cleaning', 'green_spaces' added by migration 0004
CREATE TYPE contract_type AS ENUM (
    'maintenance', 'insurance', 'service', 'other',
    'energy', 'water', 'cleaning', 'green_spaces'
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

-- allocation_key_type added by migration 0003
CREATE TYPE allocation_key_type AS ENUM ('general', 'special');

-- meeting_document_kind added by migration 0018; 'vote_by_mail_form' added by 0025
CREATE TYPE meeting_document_kind AS ENUM (
    'notice',
    'attachment',
    'vote_by_mail_form'
);

-- meeting_attendance_mode added by migration 0018
CREATE TYPE meeting_attendance_mode AS ENUM (
    'present',
    'proxy',
    'correspondence',
    'absent'
);

-- meeting_convocation_channel added by migration 0023
CREATE TYPE meeting_convocation_channel AS ENUM (
    'lrar',          -- Lettre recommandée avec accusé de réception
    'hand_delivery', -- Remise en main propre contre émargement
    'email'          -- Notification électronique (accord exprès requis — art. 64-2)
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
    shares_total    INTEGER NOT NULL,              -- base 1000 or 10000
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
-- Columns key_type and deleted_at added by migration 0003
-- ============================================================

CREATE TABLE allocation_key (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id     UUID NOT NULL REFERENCES building(id),
    name            TEXT NOT NULL,               -- e.g. "Charges Générales", "Elevator"
    description     TEXT,
    key_type        allocation_key_type NOT NULL DEFAULT 'special',
    deleted_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (building_id, name)
);

-- Enforce at most one active general key per building (migration 0003)
CREATE UNIQUE INDEX allocation_key_one_general
    ON allocation_key (building_id)
    WHERE key_type = 'general' AND deleted_at IS NULL;

-- Versioned tantiemes (migration 0002 — ADR-006)
-- valid_from / valid_to enable point-in-time queries
CREATE TABLE unit_allocation_key (
    id          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    unit_id     UUID        NOT NULL REFERENCES unit(id),
    key_id      UUID        NOT NULL REFERENCES allocation_key(id),
    share_value INTEGER     NOT NULL CHECK (share_value >= 0),
    valid_from  DATE        NOT NULL DEFAULT CURRENT_DATE,
    valid_to    DATE,                   -- NULL = current active record
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- At most one active record per (unit, key) at any time
CREATE UNIQUE INDEX uak_unique_active
    ON unit_allocation_key (unit_id, key_id)
    WHERE valid_to IS NULL;

-- ============================================================
-- USER ACCOUNTS (utilisateur)
-- phone added by migration 0007
-- ============================================================

CREATE TABLE user_account (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email               TEXT NOT NULL UNIQUE,
    last_name           TEXT NOT NULL,
    first_name          TEXT NOT NULL,
    phone               TEXT,
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
-- Session roles (president, secretary, scrutineers) added by migration 0021
-- minutes_notified_at added by migration 0022
-- convocation_sent_at / convocation_channel added by migration 0023
-- minutes signature columns added by migration 0024
-- ============================================================

CREATE TABLE general_meeting (
    id                          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id                 UUID NOT NULL REFERENCES building(id),
    meeting_type                meeting_type NOT NULL,
    status                      meeting_status NOT NULL DEFAULT 'draft',
    notice_date                 DATE,
    meeting_date                TIMESTAMPTZ NOT NULL,
    location                    TEXT,
    agenda                      TEXT,
    minutes_document_id         UUID REFERENCES document(id),
    -- Session roles (Pattern A — frozen on status = 'closed')
    president_name              TEXT,
    president_user_id           UUID REFERENCES user_account(id),
    secretary_name              TEXT,
    secretary_user_id           UUID REFERENCES user_account(id),
    -- scrutineers shape: [{ "name": "Jean Dupont", "user_id": "uuid-or-null" }, ...]
    scrutineers                 JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- Notification PV aux absents — Art. 42 loi 1965
    minutes_notified_at         TIMESTAMPTZ,
    -- Preuve d'envoi de la convocation — art. 9 décret 1967
    convocation_sent_at         TIMESTAMPTZ,
    convocation_channel         meeting_convocation_channel,
    -- Signatures du bureau de séance — art. 17 et 17-1 décret 1967
    minutes_president_signed_at TIMESTAMPTZ,
    minutes_secretary_signed_at TIMESTAMPTZ,
    -- minutes_scrutineers_signed shape: [{ "name": "Jean Dupont", "signed_at": "2026-05-13T18:00:00Z" }, ...]
    minutes_scrutineers_signed  JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_by                  UUID NOT NULL REFERENCES user_account(id),
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Meeting documents (notice, attachment, vote_by_mail_form) — migration 0018 + 0025
CREATE TABLE general_meeting_document (
    meeting_id  UUID NOT NULL REFERENCES general_meeting(id),
    document_id UUID NOT NULL REFERENCES document(id),
    kind        meeting_document_kind NOT NULL,
    PRIMARY KEY (meeting_id, document_id)
);

-- Attendance register — migration 0018
CREATE TABLE general_meeting_attendance (
    meeting_id          UUID NOT NULL REFERENCES general_meeting(id),
    unit_id             UUID NOT NULL REFERENCES unit(id),
    mode                meeting_attendance_mode NOT NULL,
    representative_name TEXT,
    shares_represented  INTEGER NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (meeting_id, unit_id)
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
    shares_represented  INTEGER NOT NULL,
    proxy_for           UUID REFERENCES user_account(id),  -- NULL = direct vote
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (resolution_id, unit_id)              -- one vote per unit per resolution
);

-- ============================================================
-- BUDGET & LEVY CALLS (ADR-009: accounting immutability)
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

-- Header record per generation batch (ADR-009)
CREATE TABLE levy_call (
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    budget_id         UUID NOT NULL REFERENCES budget(id),
    call_label        TEXT NOT NULL CHECK (length(trim(call_label)) > 0),
    due_date          DATE NOT NULL,
    charge_amount     NUMERIC(12, 2) NOT NULL DEFAULT 0,
    works_fund_amount NUMERIC(12, 2) NOT NULL DEFAULT 0,
    issued_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Aggregate total per (levy_call, budget_line) — immutable after INSERT
CREATE TABLE levy_call_line (
    levy_call_id    UUID NOT NULL REFERENCES levy_call(id) ON DELETE CASCADE,
    budget_line_id  UUID NOT NULL REFERENCES budget_line(id),
    amount          NUMERIC(12, 2) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (levy_call_id, budget_line_id)
);

-- Per-unit, per-budget-line breakdown with status (Pattern A — ADR-009)
CREATE TABLE levy_call_unit (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    levy_call_id          UUID NOT NULL REFERENCES levy_call(id) ON DELETE CASCADE,
    unit_id               UUID NOT NULL REFERENCES unit(id),
    budget_line_id        UUID REFERENCES budget_line(id),  -- NULL = works_fund portion
    key_id                UUID REFERENCES allocation_key(id),
    amount                NUMERIC(12, 2) NOT NULL,
    shares_snapshot       INTEGER,       -- tantièmes du lot — immutable after INSERT
    shares_total_snapshot INTEGER,       -- total tantièmes — immutable after INSERT
    status                levy_status NOT NULL DEFAULT 'issued',
    paid_at               TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- One works_fund row per (levy_call, unit)
CREATE UNIQUE INDEX idx_levy_call_unit_wf
    ON levy_call_unit (levy_call_id, unit_id)
    WHERE budget_line_id IS NULL;

-- One charge row per (levy_call, unit, budget_line)
CREATE UNIQUE INDEX idx_levy_call_unit_line
    ON levy_call_unit (levy_call_id, unit_id, budget_line_id)
    WHERE budget_line_id IS NOT NULL;

-- Works fund — Art. 14-2 ALUR (mandatory separate bank account)
CREATE TABLE works_fund (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id         UUID NOT NULL REFERENCES building(id),
    fiscal_year         INTEGER NOT NULL,
    levied_amount           NUMERIC(12, 2) NOT NULL DEFAULT 0,
    available_amount        NUMERIC(12, 2) NOT NULL DEFAULT 0,
    contribution_rate_pct   NUMERIC(5, 2) NOT NULL DEFAULT 5.00,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
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
-- CONTRACTOR REFERENTIAL (migration 0007 + 0008)
-- domain (single TEXT) replaced by domains (TEXT[]) in migration 0008
-- notes added by migration 0008
-- ============================================================

CREATE TABLE contractor (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id UUID NOT NULL REFERENCES building(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    email       TEXT,
    phone       TEXT,
    domains     TEXT[] NOT NULL DEFAULT '{}', -- professional domains (replaces single domain column)
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- TASKS & VENDORS (tache / prestataire)
-- contractor_id added by migration 0007
-- completion_cost added by migration 0012
-- contractor_name (free text fallback) added by migration 0013
-- ============================================================

CREATE TABLE task (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id      UUID NOT NULL REFERENCES building(id),
    title            TEXT NOT NULL,
    description      TEXT,
    status           task_status NOT NULL DEFAULT 'open',
    priority         task_priority NOT NULL DEFAULT 'normal',
    created_by       UUID NOT NULL REFERENCES user_account(id),
    assigned_to      UUID REFERENCES user_account(id),
    contract_id      UUID REFERENCES vendor_contract(id),
    contractor_id    UUID REFERENCES contractor(id) ON DELETE SET NULL,
    contractor_name  TEXT,                      -- free-text fallback when no referential entry
    completion_cost  NUMERIC(12, 2),
    vendor_token     UUID UNIQUE,               -- ephemeral UUID generated on demand
    token_expires_at TIMESTAMPTZ,
    closed_at        TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- task_id made nullable and FK changed to ON DELETE SET NULL by migration 0012
CREATE TABLE vendor_submission (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_id      UUID REFERENCES task(id) ON DELETE SET NULL,
    comment      TEXT,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Both FKs changed to ON DELETE CASCADE by migration 0017
CREATE TABLE vendor_submission_document (
    submission_id   UUID NOT NULL REFERENCES vendor_submission(id) ON DELETE CASCADE,
    document_id     UUID NOT NULL REFERENCES document(id) ON DELETE CASCADE,
    PRIMARY KEY (submission_id, document_id)
);

-- Task documents managed by the manager (migration 0017)
CREATE TABLE task_document (
    task_id     UUID NOT NULL REFERENCES task(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES document(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, document_id)
);

-- ============================================================
-- CLAIMS (sinistre)
-- claim_type, published_at, transmitted_at added by migration 0014
-- reimbursement_date, reimbursement_amount added by migration 0015
-- claim_document FKs changed to ON DELETE CASCADE by migration 0015
-- ============================================================

CREATE TABLE claim (
    id                   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id          UUID NOT NULL REFERENCES building(id),
    title                TEXT NOT NULL,
    description          TEXT,
    declared_at          DATE NOT NULL,
    status               claim_status NOT NULL DEFAULT 'open',
    claim_type           claim_type NOT NULL DEFAULT 'current',
    unit_id              UUID REFERENCES unit(id),   -- nullable for common areas
    published_at         DATE,
    transmitted_at       DATE,
    reimbursement_date   DATE,
    reimbursement_amount NUMERIC(12, 2),
    created_by           UUID NOT NULL REFERENCES user_account(id),
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT claim_published_at_by_type_check
        CHECK (
            (claim_type = 'natural_disaster' AND published_at IS NOT NULL)
            OR (claim_type <> 'natural_disaster' AND published_at IS NULL)
        ),
    CONSTRAINT claim_transmitted_after_declared_check
        CHECK (transmitted_at IS NULL OR transmitted_at >= declared_at),
    CONSTRAINT claim_reimbursement_amount_non_negative
        CHECK (reimbursement_amount IS NULL OR reimbursement_amount > 0)
);

CREATE TABLE claim_document (
    claim_id    UUID NOT NULL REFERENCES claim(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES document(id) ON DELETE CASCADE,
    PRIMARY KEY (claim_id, document_id)
);

-- ============================================================
-- MAINTENANCE LOG (carnet_entretien)
-- task_id FK changed: 0009 → ON DELETE CASCADE; 0012 → ON DELETE SET NULL (final)
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
    task_id             UUID REFERENCES task(id) ON DELETE SET NULL,
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
-- task_id (ON DELETE SET NULL) added by migration 0010
-- report_document report_id FK → ON DELETE CASCADE (migration 0010)
-- report_document document_id FK → ON DELETE CASCADE (migration 0011)
-- ============================================================

CREATE TABLE report (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id UUID NOT NULL REFERENCES building(id),
    unit_id     UUID REFERENCES unit(id),
    author_id   UUID NOT NULL REFERENCES user_account(id),
    title       TEXT NOT NULL,
    description TEXT,
    status      report_status NOT NULL DEFAULT 'open',
    task_id     UUID REFERENCES task(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE report_document (
    report_id   UUID NOT NULL REFERENCES report(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES document(id) ON DELETE CASCADE,
    PRIMARY KEY (report_id, document_id)
);

-- ============================================================
-- CONSULTATIONS / POLLS
-- consultation_response.choice_id dropped by migration 0006
-- consultation_response_choice junction table added by migration 0006
-- ============================================================

CREATE TABLE consultation (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id      UUID NOT NULL REFERENCES building(id),
    title            TEXT NOT NULL,
    description      TEXT,
    status           poll_status NOT NULL DEFAULT 'draft',
    visible_to_roles user_role[] NOT NULL,      -- who can respond
    created_by       UUID NOT NULL REFERENCES user_account(id),
    closed_at        TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE consultation_choice (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID NOT NULL REFERENCES consultation(id),
    label           TEXT NOT NULL,
    order_index     INTEGER NOT NULL DEFAULT 0
);

-- choice_id removed in migration 0006; selections live in consultation_response_choice
CREATE TABLE consultation_response (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID NOT NULL REFERENCES consultation(id),
    user_id         UUID NOT NULL REFERENCES user_account(id),
    comment         TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (consultation_id, user_id)           -- one response per person
);

-- Multi-choice junction table (migration 0006)
CREATE TABLE consultation_response_choice (
    response_id UUID NOT NULL REFERENCES consultation_response(id) ON DELETE CASCADE,
    choice_id   UUID NOT NULL REFERENCES consultation_choice(id),
    PRIMARY KEY (response_id, choice_id)
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
CREATE INDEX idx_levy_call_budget           ON levy_call(budget_id);
CREATE INDEX idx_levy_call_line_budget_line ON levy_call_line(budget_line_id);
CREATE INDEX idx_levy_call_unit_levy_call   ON levy_call_unit(levy_call_id);
CREATE INDEX idx_levy_call_unit_unit        ON levy_call_unit(unit_id);
CREATE INDEX idx_levy_call_unit_status      ON levy_call_unit(status) WHERE status != 'paid';
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

-- Contractor (migration 0007)
CREATE INDEX idx_contractor_building          ON contractor(building_id);
CREATE UNIQUE INDEX idx_contractor_building_name ON contractor(building_id, lower(name));

-- Claims (migration 0014)
CREATE INDEX idx_claim_building_status         ON claim(building_id, status);
CREATE INDEX idx_claim_building_type           ON claim(building_id, claim_type);
CREATE INDEX idx_claim_building_transmitted_at ON claim(building_id, transmitted_at);

-- Document back-links (migration 0016)
CREATE INDEX IF NOT EXISTS idx_contract_document_document_id ON contract_document(document_id);
CREATE INDEX IF NOT EXISTS idx_report_document_document_id   ON report_document(document_id);
CREATE INDEX IF NOT EXISTS idx_claim_document_document_id    ON claim_document(document_id);

-- Task documents (migration 0017)
CREATE INDEX IF NOT EXISTS idx_task_document_document_id ON task_document(document_id);

-- Meeting documents & attendance (migration 0018)
CREATE INDEX idx_general_meeting_document_document_id   ON general_meeting_document(document_id);
CREATE INDEX idx_general_meeting_attendance_meeting_id  ON general_meeting_attendance(meeting_id);

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

CREATE TRIGGER trg_consultation_updated_at
    BEFORE UPDATE ON consultation
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_works_fund_updated_at
    BEFORE UPDATE ON works_fund
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
