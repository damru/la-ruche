-- EP-005-S10: outbox-lite for document events (PRD US-DOC-009).
-- Written in the SAME transaction as the document insert; consumers poll
-- (jobs/) and mark rows processed. Real notification delivery is out of
-- scope (EPICS "hors scope") — the MVP consumer only logs.

CREATE TABLE document_event (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id   UUID NOT NULL REFERENCES building(id),
    document_id   UUID NOT NULL REFERENCES document(id),
    event_type    TEXT NOT NULL,                -- 'uploaded' (extensible: 'replaced', 'deleted')
    actor_id      UUID REFERENCES user_account(id),
    payload       JSONB NOT NULL DEFAULT '{}',  -- {original_name, category, visible_roles}
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at  TIMESTAMPTZ
);

CREATE INDEX document_event_pending_idx ON document_event (created_at) WHERE processed_at IS NULL;
