-- EP-019-S03: server-side notification preferences (US-PP-004).
-- One row per (user, optional event). Mandatory events are never stored:
-- they are always on, enforced by the domain catalog. Absent row = default.
CREATE TABLE user_notification_preference (
    user_id    UUID NOT NULL REFERENCES user_account(id) ON DELETE CASCADE,
    event_key  TEXT NOT NULL,
    enabled    BOOLEAN NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, event_key)
);
