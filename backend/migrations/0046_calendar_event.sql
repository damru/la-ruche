-- EP-014-S03: manual calendar events (PRD Calendrier §3, US-CAL-005)

CREATE TYPE calendar_event_category AS ENUM ('reunion', 'visite', 'permanence', 'autre');

CREATE TABLE calendar_event (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id  UUID NOT NULL REFERENCES building(id),
    title        TEXT NOT NULL,
    description  TEXT,
    event_date   DATE NOT NULL,
    event_time   TIME,                          -- heure optionnelle (PRD §3)
    category     calendar_event_category,       -- optionnelle (AC-CAL-005-B)
    created_by   UUID NOT NULL REFERENCES user_account(id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_calendar_event_building_date ON calendar_event (building_id, event_date);

-- AC-CAL-005-C: attachments live in the GED (document) and are linked here,
-- mirroring maintenance_log_document / task_document.
CREATE TABLE calendar_event_document (
    event_id    UUID NOT NULL REFERENCES calendar_event(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES document(id) ON DELETE CASCADE,
    PRIMARY KEY (event_id, document_id)
);
