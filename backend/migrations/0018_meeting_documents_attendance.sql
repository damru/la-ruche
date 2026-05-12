CREATE TYPE meeting_document_kind AS ENUM (
    'notice',
    'attachment'
);

CREATE TYPE meeting_attendance_mode AS ENUM (
    'present',
    'proxy',
    'correspondence',
    'absent'
);

CREATE TABLE general_meeting_document (
    meeting_id UUID NOT NULL REFERENCES general_meeting(id),
    document_id UUID NOT NULL REFERENCES document(id),
    kind meeting_document_kind NOT NULL,
    PRIMARY KEY (meeting_id, document_id)
);

CREATE TABLE general_meeting_attendance (
    meeting_id UUID NOT NULL REFERENCES general_meeting(id),
    unit_id UUID NOT NULL REFERENCES unit(id),
    mode meeting_attendance_mode NOT NULL,
    representative_name TEXT,
    shares_represented INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (meeting_id, unit_id)
);

CREATE INDEX idx_general_meeting_document_document_id ON general_meeting_document(document_id);
CREATE INDEX idx_general_meeting_attendance_meeting_id ON general_meeting_attendance(meeting_id);