-- EP-010-S05: reminder log for consultation non-respondents (US-CS-004).
-- Delivery infrastructure does not exist yet; this table records the intent
-- (who triggered a reminder, when, how many recipients) and will feed the
-- future notification dispatcher.
CREATE TABLE consultation_reminder (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID NOT NULL REFERENCES consultation(id) ON DELETE CASCADE,
    sent_by         UUID NOT NULL REFERENCES user_account(id),
    recipient_count INTEGER NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX consultation_reminder_consultation_idx
    ON consultation_reminder (consultation_id);
