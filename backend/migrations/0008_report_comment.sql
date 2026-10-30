-- Append-only timestamped follow-up comments on reports (US-SIG-006).
-- No updated_at on purpose: rows are never updated nor deleted;
-- they are removed only via ON DELETE CASCADE when the report is deleted.
CREATE TABLE report_comment (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id   UUID NOT NULL REFERENCES report(id) ON DELETE CASCADE,
    author_id   UUID NOT NULL REFERENCES user_account(id),
    body        TEXT NOT NULL CHECK (length(trim(body)) > 0),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX report_comment_report_idx ON report_comment (report_id, created_at);
