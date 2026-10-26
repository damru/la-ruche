-- Append-only timestamped follow-up notes on claims.
-- No updated_at on purpose: rows are never updated nor deleted.
CREATE TABLE claim_note (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    claim_id    UUID NOT NULL REFERENCES claim(id) ON DELETE CASCADE,
    author_id   UUID NOT NULL REFERENCES user_account(id),
    body        TEXT NOT NULL CHECK (length(trim(body)) > 0),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX claim_note_claim_idx ON claim_note (claim_id, created_at);
